// ID interpolation ported and modified from swc-project/plugins (Apache-2.0).
import { createHash } from 'node:crypto';
import path from 'node:path';

export function interpolate(pattern, content, filename) {
  const parsed = path.posix.parse(filename.replaceAll('\\', '/'));
  let directory = (parsed.dir ? `${parsed.dir}/` : filename).replace(/\.\.(\/)?/g, '_$1');
  if (directory.length <= 1) { directory = ''; }
  const placeholders = {
    content, ext: parsed.ext.slice(1) || 'bin', name: parsed.dir ? parsed.name : 'file',
    path: directory, folder: path.posix.basename(directory), query: '',
  };
  // Replace in one pass: message content can itself contain [name] or [contenthash].
  return pattern.replace(/\[(?:([^:\]]+):)?(?:hash|contenthash)(?::([a-z][a-z0-9]*))?(?::(\d+))?\]|\[(content|ext|name|path|folder|query)\]/g,
    (match, algorithm, encoding, length, placeholder) => {
      if (placeholder) { return placeholders[placeholder]; }
      const hash = ['md5', 'sha1'].includes(algorithm) ? algorithm : 'sha512';
      const format = ['base64', 'base64url'].includes(encoding) ? encoding : 'hex';
      const digest = createHash(hash).update(content).digest(format);
      return length === undefined ? digest : digest.slice(0, Number(length));
    });
}
