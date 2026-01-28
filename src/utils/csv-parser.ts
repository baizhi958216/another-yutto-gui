import type { Comment } from '@/types'

/**
 * Parse CSV content into Comment objects
 * Handles quoted fields and escaped quotes
 */
export function parseCsvComments(csvContent: string): Comment[] {
  const lines = csvContent.trim().split('\n')
  if (lines.length <= 1) {
    return []
  }

  // Skip header line
  const dataLines = lines.slice(1)
  const parsedComments: Comment[] = []

  for (const line of dataLines) {
    try {
      // Simple CSV parsing (handles quoted fields)
      const fields: string[] = []
      let currentField = ''
      let inQuotes = false

      for (let i = 0; i < line.length; i++) {
        const char = line[i]

        if (char === '"') {
          if (inQuotes && line[i + 1] === '"') {
            // Escaped quote
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

      // CSV format: rpid,oid,mid,uname,sex,content,avatar,ctime,like,level,location,parent
      if (fields.length >= 12) {
        parsedComments.push({
          rpid: Number.parseInt(fields[0]),
          oid: Number.parseInt(fields[1]),
          mid: Number.parseInt(fields[2]),
          uname: fields[3],
          sex: fields[4],
          content: fields[5],
          avatar: fields[6],
          ctime: Number.parseInt(fields[7]),
          like: Number.parseInt(fields[8]),
          current_level: Number.parseInt(fields[9]),
          location: fields[10],
          parent: Number.parseInt(fields[11]),
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
