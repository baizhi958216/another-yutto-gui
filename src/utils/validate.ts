/**
 * 验证 B 站 URL 或视频 ID
 * 支持 yutto 所有支持的链接格式
 * @param url 待验证的 URL 或视频 ID
 * @returns 是否为有效的 B 站 URL 或视频 ID
 */
export function validateBilibiliUrl(url: string): boolean {
  const patterns = [
    // 投稿视频
    /^https?:\/\/(www\.)?bilibili\.com\/video\/BV\w+/,
    /^https?:\/\/(www\.)?bilibili\.com\/video\/av\d+/,
    /^BV[a-zA-Z0-9]+$/, // 纯 BV 号
    /^av\d+$/i, // 纯 AV 号（支持大小写）

    // 番剧
    /^https?:\/\/(www\.)?bilibili\.com\/bangumi\/play\/ep\d+/,
    /^https?:\/\/(www\.)?bilibili\.com\/bangumi\/play\/ss\d+/,
    /^https?:\/\/(www\.)?bilibili\.com\/bangumi\/media\/md\d+/,
    /^ep\d+$/i, // 纯 EP 号
    /^ss\d+$/i, // 纯 SS 号
    /^md\d+$/i, // 纯 MD 号

    // 课程
    /^https?:\/\/(www\.)?bilibili\.com\/cheese\/play\/ep\d+/,
    /^https?:\/\/(www\.)?bilibili\.com\/cheese\/play\/ss\d+/,

    // 短链接
    /^https?:\/\/b23\.tv\//,

    // 收藏夹
    /^https?:\/\/space\.bilibili\.com\/\d+\/favlist/,

    // 稍后再看
    /^https?:\/\/(www\.)?bilibili\.com\/(watchlater|list\/watchlater)/,

    // UP 主空间
    /^https?:\/\/space\.bilibili\.com\/\d+\/video/,

    // 合集和列表
    /^https?:\/\/space\.bilibili\.com\/\d+\/lists/,
    /^https?:\/\/(www\.)?bilibili\.com\/list\/\d+/,
  ]
  return patterns.some(pattern => pattern.test(url))
}

/**
 * 规范化 B 站 URL
 * 将纯视频 ID 转换为完整的 URL
 * @param url 待规范化的 URL 或视频 ID
 * @returns 规范化后的 URL
 */
export function normalizeBilibiliUrl(url: string): string {
  // 如果是纯 BV 号，转换为完整 URL
  if (/^BV[a-zA-Z0-9]+$/.test(url)) {
    return `https://www.bilibili.com/video/${url}`
  }

  // 如果是纯 AV 号，转换为完整 URL
  if (/^av\d+$/i.test(url)) {
    return `https://www.bilibili.com/video/${url.toLowerCase()}`
  }

  // 如果是纯 EP 号，转换为完整 URL
  if (/^ep\d+$/i.test(url)) {
    return `https://www.bilibili.com/bangumi/play/${url.toLowerCase()}`
  }

  // 如果是纯 SS 号，转换为完整 URL
  if (/^ss\d+$/i.test(url)) {
    return `https://www.bilibili.com/bangumi/play/${url.toLowerCase()}`
  }

  // 如果是纯 MD 号，转换为完整 URL
  if (/^md\d+$/i.test(url)) {
    return `https://www.bilibili.com/bangumi/media/${url.toLowerCase()}`
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
 * 验证视频质量值
 * @param quality 质量值
 * @returns 是否为有效的质量值
 */
export function validateQuality(quality: number): boolean {
  const validQualities = [16, 32, 64, 80, 112, 116, 120, 125, 126, 127]
  return validQualities.includes(quality)
}
