#!/usr/bin/env node
/* eslint-disable node/prefer-global/process */

import { spawn } from 'node:child_process'
import { platform } from 'node:process'

/**
 * Execute a command and return a promise
 */
function execCommand(command, args, options = {}) {
  return new Promise((resolve, reject) => {
    console.log(`\n> ${command} ${args.join(' ')}`)

    const child = spawn(command, args, {
      stdio: 'inherit',
      shell: true,
      ...options,
    })

    child.on('close', (code) => {
      if (code !== 0) {
        reject(new Error(`Command failed with exit code ${code}: ${command} ${args.join(' ')}`))
      }
      else {
        resolve()
      }
    })

    child.on('error', (error) => {
      reject(error)
    })
  })
}

async function buildWindows() {
  console.log('Building for Windows...\n')

  try {
    // Step 1: Download ffmpeg
    console.log('Step 1/4: Downloading ffmpeg...')
    await execCommand('node', ['build-scripts/download-ffmpeg.js'])

    // Step 2: Build yutto (must run from build-scripts directory)
    console.log('\nStep 2/4: Building yutto...')
    await execCommand('powershell', ['-ExecutionPolicy', 'Bypass', '-File', './build-yutto.ps1'], {
      cwd: 'build-scripts',
    })

    // Step 3: Build Tauri app
    console.log('\nStep 3/4: Building Tauri app...')
    await execCommand('npm', ['run', 'tauri', 'build'])

    console.log('\n✓ Build completed successfully!')
  }
  catch (error) {
    console.error('\n✗ Build failed:', error.message)
    process.exit(1)
  }
}

async function buildUnix() {
  console.log('Building for Unix-like system...\n')

  try {
    // Step 1: Download ffmpeg
    console.log('Step 1/4: Downloading ffmpeg...')
    await execCommand('node', ['build-scripts/download-ffmpeg.js'])

    // Step 2: Build yutto (must run from build-scripts directory)
    console.log('\nStep 2/4: Building yutto...')
    await execCommand('bash', ['./build-yutto.sh'], {
      cwd: 'build-scripts',
    })

    // Step 3: Build Tauri app
    console.log('\nStep 3/4: Building Tauri app...')
    await execCommand('npm', ['run', 'tauri', 'build'])

    console.log('\n✓ Build completed successfully!')
  }
  catch (error) {
    console.error('\n✗ Build failed:', error.message)
    process.exit(1)
  }
}

async function main() {
  console.log(`Detected platform: ${platform}\n`)

  if (platform === 'win32') {
    await buildWindows()
  }
  else {
    await buildUnix()
  }
}

main().catch((error) => {
  console.error('Fatal error:', error)
  process.exit(1)
})
