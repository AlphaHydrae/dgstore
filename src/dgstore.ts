import * as glob from 'fast-glob';
import { readFile, Stats, writeFile } from 'fs-extra';

import { Digest } from './digest';
import { DigestFile } from './digest-file';
import { EventEmitter } from './event-emitter';
import { FileDigest } from './file-digest';
import { FileResult } from './file-result';
import { hash } from './hash';

export interface DgstoreEvents {
  hashStart: GlobStats;
  hashEnd: FileResult;
  scanStart: undefined;
  scanEnd: GlobStats[];
}

export interface DgstoreOptions {
  fullDigest?: boolean;
  events?: EventEmitter<DgstoreEvents>;
  targets: string[];
  write?: boolean;
}

export interface GlobStats extends Stats {
  path: string;
}

export async function dgstore(options: DgstoreOptions) {

  const events = options.events || new EventEmitter();

  events.emit('scanStart', undefined);
  const filesAndDigests = await glob<GlobStats>(options.targets, { stats: true });

  const stats = filesAndDigests.filter((current: GlobStats) => !current.path.match(/\.sha512$/) || !filesAndDigests.find(other => other.path === current.path.replace(/\.sha512$/, '')));
  events.emit('scanEnd', stats);

  const results: FileResult[] = [];

  // TODO: parallelize
  for (const stat of stats) {

    events.emit('hashStart', stat);

    const result = await compareOrStoreDigest(stat, options);
    results.push(result);

    events.emit('hashEnd', result);
  }

  return results;
}

export async function compareOrStoreDigest(stats: GlobStats, options: DgstoreOptions) {

  const previousDigest = await readDigest(stats);
  const digest = await hash(stats.path);

  let digestFile;
  if (!previousDigest && options.write) {
    await storeDigest(stats, digest);
    digestFile = new DigestFile(`${stats.path}.sha512`, digest, true);
  } else if (previousDigest) {
    digestFile = new DigestFile(`${stats.path}.sha512`, previousDigest, false);
  }

  return new FileResult(new FileDigest(stats.path, digest), digestFile);
}

async function readDigest(file: GlobStats) {
  try {
    const contents = await readFile(`${file.path}.sha512`, 'utf8');
    return new Digest(Buffer.from(contents.trim(), 'hex'));
  } catch (err) {
    if (err.code !== 'ENOENT') {
      throw err;
    }

    return;
  }
}

async function storeDigest(stats: GlobStats, digest: Digest) {
  await writeFile(`${stats.path}.sha512`, digest.hex, 'utf8');
}
