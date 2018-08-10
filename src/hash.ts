import { createHash } from 'crypto';
import { createReadStream, ReadStream } from 'fs-extra';

import { Digest } from './digest';

export function hash(file: string): Promise<Digest> {
  return new Promise((resolve, reject) => createReadStream(file)
    .on('error', reject)
    .pipe(createHash('sha512'))
    .once('finish', function(this: ReadStream) {
      resolve(new Digest(this.read()));
    }));
}
