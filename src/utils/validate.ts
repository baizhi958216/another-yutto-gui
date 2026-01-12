/**
 * 验证 B 站 URL 或 BV 号
 * @param url 待验证的 URL 或 BV 号
 * @returns 是否为有效的 B 站 URL 或 BV 号
 */
export function validateBilibiliUrl(url: string): boolean {
  const patterns = [
    /^https?:\/\/(www\.)?bilibili\.com\/video\/BV\w+/,
    /^https?:\/\/(www\.)?bilibili\.com\/bangumi\/play\/ep\d+/,
    /^https?:\/\/(www\.)?bilibili\.com\/bangumi\/play\/ss\d+/,
    /^BV[a-zA-Z0-9]+$/, // 支持纯 BV 号
  ]
  return patterns.some(pattern => pattern.test(url))
}

/**
 * 规范化 B 站 URL
 * 如果输入是 BV 号，转换为完整的 URL
 * @param url 待规范化的 URL 或 BV 号
 * @returns 规范化后的 URL
 */
export function normalizeBilibiliUrl(url: string): string {
  // 如果是纯 BV 号，转换为完整 URL
  if (/^BV[a-zA-Z0-9]+$/.test(url)) {
    return `https://www.bilibili.com/video/${url}`
  }
  return url
}

/**
 * 验证路径
 * @param path 待验证的路径
 * @returns 是否为有效路径
 */
export function validatePath(path: string): boolean {
  if (!path || path.length === 0)
    return false
  // 检查是否包含非法字符
  const illegalChars = ['<', '>', '|', '"', '*', '?']
  return !illegalChars.some(char => path.includes(char))
}

/**
 * 验证预设名称
 * @param name 预设名称
 * @returns 是否为有效的预设名称
 */
export function validatePresetName(name: string): boolean {
  return name.length > 0 && name.length <= 50
}

/**
 * 验证视频质量值
 * @param quality 质量值
 * @returns 是否为有效的质量值
 */
export function validateQuality(quality: number): boolean {
  const validQualities = [16, 32, 64, 80, 112, 116, 120, 125, 126, 127]
  return validQualities.includes(quality)
}
