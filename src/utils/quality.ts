/**
 * 格式化视频质量显示
 * @param quality 视频质量代码
 * @returns 格式化后的视频质量字符串
 */
export function formatVideoQuality(quality: number): string {
  const qualityMap: Record<number, string> = {
    127: '8K',
    126: '杜比视界',
    125: 'HDR',
    120: '4K',
    116: '1080P60',
    112: '1080P+',
    100: '智能修复',
    80: '1080P',
    74: '720P60',
    64: '720P',
    32: '480P',
    16: '360P',
  }
  return qualityMap[quality] || `${quality}P`
}

/**
 * 格式化音频质量显示
 * @param quality 音频质量代码
 * @returns 格式化后的音频质量字符串
 */
export function formatAudioQuality(quality: number): string {
  const qualityMap: Record<number, string> = {
    30251: 'Hi-Res',
    30255: '杜比音效',
    30250: '杜比全景声',
    30280: '320K',
    30232: '132K',
    30216: '64K',
  }
  return qualityMap[quality] || `${quality}`
}

/**
 * 格式化完整的质量信息（视频 + 音频）
 * @param videoQuality 视频质量代码
 * @param audioQuality 音频质量代码
 * @param videoOnly 是否仅视频
 * @param audioOnly 是否仅音频
 * @returns 格式化后的质量字符串
 */
export function formatQuality(
  videoQuality: number,
  audioQuality: number,
  videoOnly?: boolean,
  audioOnly?: boolean,
): string {
  if (audioOnly) {
    return `仅音频 ${formatAudioQuality(audioQuality)}`
  }
  if (videoOnly) {
    return `仅视频 ${formatVideoQuality(videoQuality)}`
  }
  return `${formatVideoQuality(videoQuality)} + ${formatAudioQuality(audioQuality)}`
}

/**
 * 获取质量选项的显示文本（包含可用性提示）
 * @param quality 质量选项对象
 * @param quality.description 质量描述
 * @param quality.available 质量是否存在
 * @param quality.vip_only 是否仅限vip
 * @param quality.login_required 是否需要登录
 * @returns 带有可用性提示的显示文本
 */
export function getQualityDisplayText(quality: {
  description: string
  available?: boolean
  vip_only?: boolean
  login_required?: boolean
}): string {
  // 如果 available 未定义，默认为 true（可用）
  const isAvailable = quality.available !== false
  if (isAvailable) {
    return quality.description
  }
  if (quality.vip_only) {
    return `${quality.description} (需要大会员)`
  }
  if (quality.login_required) {
    return `${quality.description} (需要登录)`
  }
  return quality.description
}
