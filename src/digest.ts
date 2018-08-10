export class Digest {
  private readonly cached: { [key: string]: string };

  constructor(readonly value: Buffer) {
    this.cached = {};
  }

  equals(other: Digest) {
    return this.value.equals(other.value);
  }

  get hex() {
    return this.getCached('hex', () => this.value.toString('hex'));
  }

  private getCached(key: string, factory: () => string) {
    if (!this.cached[key]) {
      this.cached[key] = factory();
    }

    return this.cached[key];
  }
}
