import { Digest } from './digest';

export class FileDigest {
  constructor(readonly file: string, readonly digest: Digest) {
  }
}
