import { isInteger, isObject } from 'lodash';

import { Digest } from './digest';

export const DEFAULT_DIGEST_MIN_LENGTH = 6;

export interface DigestShortenerOptions {
  fullLength?: boolean;
  minLength?: number;
}

export class DigestShortener {
  readonly fullLength: boolean;
  readonly minLength: number;
  private readonly knownDigests: string[];

  constructor(options: DigestShortenerOptions = {}) {
    if (!isObject(options)) {
      throw new Error(`Options must be an object; got type ${typeof(options)}`);
    } else if (options.minLength !== undefined && (!isInteger(options.minLength) || options.minLength < 0)) {
      throw new Error(`"minLength" option must be an integer greater than or equal to one; got ${JSON.stringify(options.minLength)}`);
    }

    this.fullLength = Boolean(options.fullLength);
    this.minLength = options.minLength || DEFAULT_DIGEST_MIN_LENGTH;
    this.knownDigests = [];
  }

  shorten(digest: Digest, differentThan?: Digest) {

    const hex = digest.hex;
    if (this.fullLength) {
      this.knownDigests.push(hex);
      return hex;
    }

    const mustDifferFrom = this.knownDigests.slice();
    if (differentThan) {
      mustDifferFrom.push(differentThan.hex);
    }

    const length = hex.length;
    let currentLength = this.minLength;
    while (currentLength <= length) {

      const shortHex = hex.slice(0, currentLength);

      let matchingOther;
      for (const other of mustDifferFrom) {
        if (other.indexOf(shortHex) === 0) {
          matchingOther = other;
          break;
        }
      }

      if (matchingOther) {
        currentLength++;
      } else {
        const shortened = hex.slice(0, currentLength);
        this.knownDigests.push(shortened);
        return shortened;
      }
    }

    this.knownDigests.push(hex);
    return hex;
  }
}
