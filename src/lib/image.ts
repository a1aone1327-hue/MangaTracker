// Локальная обложка: уменьшаем картинку и храним прямо в базе как data URL.
// Так она едет вместе с экспортом и не требует доступа к файлам и разрешений.

const MAX_W = 240;
const MAX_H = 340;

export async function fileToThumbnail(file: Blob): Promise<string> {
  const bitmap = await createImageBitmap(file);
  const scale = Math.min(1, MAX_W / bitmap.width, MAX_H / bitmap.height);
  const w = Math.max(1, Math.round(bitmap.width * scale));
  const h = Math.max(1, Math.round(bitmap.height * scale));
  const canvas = document.createElement('canvas');
  canvas.width = w;
  canvas.height = h;
  const ctx = canvas.getContext('2d');
  if (!ctx) throw new Error('canvas недоступен');
  ctx.drawImage(bitmap, 0, 0, w, h);
  bitmap.close();
  return canvas.toDataURL('image/jpeg', 0.75);
}

/** Уменьшает картинку, пришедшую как data URL (например, скачанную обложку). */
export async function dataUrlToThumbnail(dataUrl: string): Promise<string> {
  const comma = dataUrl.indexOf(',');
  if (comma < 0) throw new Error('неверный data URL');
  const mime = /^data:([^;,]+)/.exec(dataUrl)?.[1] ?? 'image/jpeg';
  const binary = atob(dataUrl.slice(comma + 1));
  const bytes = new Uint8Array(binary.length);
  for (let i = 0; i < binary.length; i += 1) bytes[i] = binary.charCodeAt(i);
  return fileToThumbnail(new Blob([bytes], { type: mime }));
}
