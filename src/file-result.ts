import { DigestFile } from './digest-file';
import { FileDigest } from './file-digest';

export class FileResult {
  constructor(readonly fileDigest: FileDigest, readonly digestFile?: DigestFile) {
  }
}
