import type { Comment } from '@/types'

function parseCsvLine(line: string): string[] {
  const fields: string[] = []
  let currentField = ''
  let inQuotes = false

  for (let i = 0; i < line.length; i++) {
    const char = line[i]

    if (char === '"') {
      if (inQuotes && line[i + 1] === '"') {
        currentField += '"'
        i++
      }
      else {
        inQuotes = !inQuotes
      }
    }
    else if (char === ',' && !inQuotes) {
      fields.push(currentField)
      currentField = ''
    }
    else {
      currentField += char
    }
  }

  fields.push(currentField)
  return fields
}

function getField(
  fields: string[],
  headerIndexMap: Record<string, number>,
  name: string,
  fallbackIndex: number,
): string {
  const headerIndex = headerIndexMap[name]
  if (headerIndex !== undefined && headerIndex < fields.length) {
    return fields[headerIndex] || ''
  }
  return fields[fallbackIndex] || ''
}

function toNumber(value: string, defaultValue: number = 0): number {
  const parsed = Number.parseInt(value, 10)
  return Number.isNaN(parsed) ? defaultValue : parsed
}

/**
 * Parse CSV content into Comment objects
 * Handles quoted fields and escaped quotes.
 * Supports both old CSV format (12 columns) and new format (with root/reply_count).
 */
export function parseCsvComments(csvContent: string): Comment[] {
  const lines = csvContent.trim().split('\n')
  if (lines.length <= 1) {
    return []
  }

  const headerFields = parseCsvLine(lines[0]).map(field => field.trim())
  const headerIndexMap: Record<string, number> = {}

  for (const [index, name] of headerFields.entries()) {
    headerIndexMap[name] = index
  }

  const dataLines = lines.slice(1)
  const parsedComments: Comment[] = []

  for (const line of dataLines) {
    if (!line.trim()) {
      continue
    }

    try {
      const fields = parseCsvLine(line)

      if (fields.length >= 12) {
        const rpid = toNumber(getField(fields, headerIndexMap, 'rpid', 0))
        const oid = toNumber(getField(fields, headerIndexMap, 'oid', 1))
        const mid = toNumber(getField(fields, headerIndexMap, 'mid', 2))
        const uname = getField(fields, headerIndexMap, 'uname', 3)
        const sex = getField(fields, headerIndexMap, 'sex', 4)
        const content = getField(fields, headerIndexMap, 'content', 5)
        const avatar = getField(fields, headerIndexMap, 'avatar', 6)
        const ctime = toNumber(getField(fields, headerIndexMap, 'ctime', 7))
        const like = toNumber(getField(fields, headerIndexMap, 'like', 8))
        const currentLevel = toNumber(getField(fields, headerIndexMap, 'level', 9))
        const location = getField(fields, headerIndexMap, 'location', 10)
        const parent = toNumber(getField(fields, headerIndexMap, 'parent', 11))

        // New fields for threaded comment rendering.
        const rootRaw = getField(fields, headerIndexMap, 'root', 12)
        const replyCountRaw = getField(fields, headerIndexMap, 'reply_count', 13)
        const root = rootRaw ? toNumber(rootRaw) : 0
        const replyCount = replyCountRaw ? toNumber(replyCountRaw) : 0

        parsedComments.push({
          rpid,
          oid,
          mid,
          root,
          uname,
          sex,
          content,
          avatar,
          ctime,
          like,
          reply_count: replyCount,
          current_level: currentLevel,
          location,
          parent,
          pictures: [],
        })
      }
    }
    catch (error) {
      console.error('Failed to parse comment line:', error)
    }
  }

  return parsedComments
}
