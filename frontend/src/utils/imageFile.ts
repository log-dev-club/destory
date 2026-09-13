// 첨부 이미지가 원본 그대로 base64 로 마크다운에 박히면 본문 용량이 지나치게 커지므로,
// 삽입 전에 너무 큰 이미지는 리사이즈·압축해서 데이터 URI 크기를 줄인다.
const MAX_DIMENSION = 1280
const JPEG_QUALITY = 0.8

function readAsDataUrl(file: File): Promise<string> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader()
    reader.onload = () => resolve(reader.result as string)
    reader.onerror = () => reject(reader.error)
    reader.readAsDataURL(file)
  })
}

function loadImage(src: string): Promise<HTMLImageElement> {
  return new Promise((resolve, reject) => {
    const img = new Image()
    img.onload = () => resolve(img)
    img.onerror = () => reject(new Error('이미지를 불러오지 못했습니다'))
    img.src = src
  })
}

function hasTransparency(ctx: CanvasRenderingContext2D, width: number, height: number): boolean {
  const { data } = ctx.getImageData(0, 0, width, height)
  for (let i = 3; i < data.length; i += 4) {
    if (data[i] < 255) return true
  }
  return false
}

/**
 * 이미지 파일을 데이터 URI로 변환한다. 가로/세로가 MAX_DIMENSION 을 넘으면 비율을 유지한 채
 * 줄이고, 투명 배경이 필요 없는 형식은 JPEG로 다시 인코딩해 용량을 크게 줄인다.
 */
export async function fileToCompressedDataUrl(file: File): Promise<string> {
  const original = await readAsDataUrl(file)

  const canvas = document.createElement('canvas')
  const ctx = canvas.getContext('2d')
  if (!ctx) return original

  let image: HTMLImageElement
  try {
    image = await loadImage(original)
  } catch {
    return original
  }

  const scale = Math.min(1, MAX_DIMENSION / Math.max(image.width, image.height))
  canvas.width = Math.max(1, Math.round(image.width * scale))
  canvas.height = Math.max(1, Math.round(image.height * scale))
  ctx.drawImage(image, 0, 0, canvas.width, canvas.height)

  // 실제로 투명 픽셀을 쓰는 경우에만 무손실 형식을 유지한다.
  // (스크린샷 등 PNG로 저장됐지만 완전 불투명한 이미지가 많아, 이런 경우는 jpeg가 훨씬 작다)
  const mimeType = hasTransparency(ctx, canvas.width, canvas.height) ? 'image/png' : 'image/jpeg'
  const compressed = canvas.toDataURL(mimeType, mimeType === 'image/jpeg' ? JPEG_QUALITY : undefined)

  // 리사이즈·압축 결과가 오히려 더 크면(작은 이미지 등) 원본을 그대로 사용
  return compressed.length < original.length ? compressed : original
}
