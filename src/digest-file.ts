import { Digest } from './digest';

export class DigestFile {
  constructor(readonly file: string, readonly digest: Digest, readonly created: boolean) {
  }
}
