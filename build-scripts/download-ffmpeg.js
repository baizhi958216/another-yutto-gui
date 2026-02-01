#!/usr/bin/env node

import { createWriteStream, existsSync, mkdirSync, chmodSync, rmSync, renameSync } from 'node:fs'
import { join, dirname } from 'node:path'
import { fileURLToPath } from 'node:url'
import { pipeline } from 'node:stream/promises'

const __dirname = dirname(fileURLToPath(import.meta.url))
const BINARIES_DIR = join(__dirname, '..', 'binaries')

// BtbN FFmpeg-Builds for Windows and Linux
const BTBN_VERSION = 'autobuild-2026-01-31-12-57'
const BTBN_BASE_URL = `https://github.com/BtbN/FFmpeg-Builds/releases/download/${BTBN_VERSION}`

// martin-riedl.de for macOS
const MARTIN_RIEDL_BASE_URL = 'https://ffmpeg.martin-riedl.de/download/macos'

const PLATFORMS = [
  {
    url: `${BTBN_BASE_URL}/ffmpeg-N-122607-g50bcc96a75-win64-gpl.zip`,
    targetName: 'ffmpeg-x86_64-pc-windows-msvc.exe',
    archivePath: 'ffmpeg-N-122607-g50bcc96a75-win64-gpl/bin/ffmpeg.exe',
    isZip: true,
    needsExtraction: true,
  },
  {
    url: `${BTBN_BASE_URL}/ffmpeg-N-122607-g50bcc96a75-linux64-gpl.tar.xz`,
    targetName: 'ffmpeg-x86_64-unknown-linux-gnu',
    archivePath: 'ffmpeg-N-122607-g50bcc96a75-linux64-gpl/bin/ffmpeg',
    isTarXz: true,
    needsExtraction: true,
  },
  {
    url: `${MARTIN_RIEDL_BASE_URL}/amd64/1767299902_N-122320-g38e89fe502/ffmpeg.zip`,
    targetName: 'ffmpeg-x86_64-apple-darwin',
    archivePath: 'ffmpeg',
    isZip: true,
    needsExtraction: true,
  },
  {
    url: `${MARTIN_RIEDL_BASE_URL}/arm64/1769883472_N-122609-g364d5dda91/ffmpeg.zip`,
    targetName: 'ffmpeg-aarch64-apple-darwin',
    archivePath: 'ffmpeg',
    isZip: true,
    needsExtraction: true,
  },
]

async function downloadFile(url, destPath) {
  console.log(`Downloading ${url}...`)
  const response = await fetch(url)
  if (!response.ok) {
    throw new Error(`Failed to download ${url}: ${response.statusText}`)
  }

  const fileStream = createWriteStream(destPath)
  await pipeline(response.body, fileStream)
  console.log(`Downloaded to ${destPath}`)
}

async function extractTarXz(archivePath, targetFile, outputPath) {
  console.log(`Extracting ${targetFile} from ${archivePath}...`)

  const tempDir = join(dirname(outputPath), `temp_extract_${Date.now()}`)
  mkdirSync(tempDir, { recursive: true })

  try {
    // Use decompress to handle tar.xz
    const decompress = (await import('decompress')).default
    const decompressTarxz = (await import('decompress-tarxz')).default

    await decompress(archivePath, tempDir, {
      plugins: [decompressTarxz()],
      filter: (file) => file.path === targetFile,
    })

    // Move the extracted file to the target location
    const extractedPath = join(tempDir, targetFile)
    if (existsSync(extractedPath)) {
      renameSync(extractedPath, outputPath)
      chmodSync(outputPath, 0o755)
    } else {
      throw new Error(`Expected file ${targetFile} not found in archive`)
    }
  } finally {
    // Clean up temp directory
    if (existsSync(tempDir)) {
      rmSync(tempDir, { recursive: true, force: true })
    }
  }
}

async function extractZip(archivePath, targetFile, outputPath) {
  console.log(`Extracting ${targetFile} from ${archivePath}...`)

  // Dynamic import for unzipper
  const unzipper = await import('unzipper')
  const { createReadStream } = await import('node:fs')

  return new Promise((resolve, reject) => {
    createReadStream(archivePath)
      .pipe(unzipper.Parse())
      .on('entry', (entry) => {
        if (entry.path === targetFile) {
          entry.pipe(createWriteStream(outputPath))
            .on('finish', () => {
              if (process.platform !== 'win32') {
                chmodSync(outputPath, 0o755)
              }
              resolve()
            })
            .on('error', reject)
        } else {
          entry.autodrain()
        }
      })
      .on('error', reject)
  })
}

async function downloadAndExtractFFmpeg(platform) {
  const outputPath = join(BINARIES_DIR, platform.targetName)

  // Skip if already exists
  if (existsSync(outputPath)) {
    console.log(`${platform.targetName} already exists, skipping...`)
    return
  }

  try {
    // For direct binary downloads (macOS x86_64 from shaka-project)
    if (!platform.needsExtraction) {
      console.log(`Downloading ${platform.url}...`)
      const response = await fetch(platform.url)
      if (!response.ok) {
        throw new Error(`Failed to download ${platform.url}: ${response.statusText}`)
      }

      const fileStream = createWriteStream(outputPath)
      await pipeline(response.body, fileStream)

      // Set executable permissions
      chmodSync(outputPath, 0o755)
      console.log(`✓ Successfully downloaded ${platform.targetName}`)
      return
    }

    // For archived downloads
    const tempArchive = join(BINARIES_DIR, `temp_${Date.now()}${platform.isZip ? '.zip' : '.tar.xz'}`)

    // Download archive
    await downloadFile(platform.url, tempArchive)

    // Extract specific file
    if (platform.isTarXz) {
      await extractTarXz(tempArchive, platform.archivePath, outputPath)
    } else if (platform.isZip) {
      await extractZip(tempArchive, platform.archivePath, outputPath)
    }

    console.log(`✓ Successfully extracted ${platform.targetName}`)

    // Clean up temp archive
    if (existsSync(tempArchive)) {
      rmSync(tempArchive, { force: true })
    }
  } catch (error) {
    console.error(`✗ Failed to process ${platform.targetName}:`, error.message)
    throw error
  }
}

async function main() {
  console.log('Starting FFmpeg download and extraction...\n')

  // Create binaries directory if it doesn't exist
  if (!existsSync(BINARIES_DIR)) {
    mkdirSync(BINARIES_DIR, { recursive: true })
  }

  // Download and extract for each platform
  for (const platform of PLATFORMS) {
    try {
      await downloadAndExtractFFmpeg(platform)
    } catch (error) {
      console.error(`Failed to process ${platform.targetName}, continuing...`)
    }
  }

  console.log('\n✓ FFmpeg download complete!')
}

main().catch((error) => {
  console.error('Fatal error:', error)
  process.exit(1)
})
