import chalk from 'chalk';
import { createHash } from 'crypto';
import * as glob from 'fast-glob';
import { createReadStream, readFile, ReadStream, Stats } from 'fs-extra';

import { program } from './program';

export function cli(argv = process.argv) {
  return Promise
    .resolve(argv)
    .then(program)
    .then(dgstore)
    .catch(err => console.error(chalk.red(err.stack)));
}

export interface DgstoreOptions {
  targets: string[];
}

export interface GlobStats extends Stats {
  path: string;
}

export async function dgstore(options: DgstoreOptions) {

  const filesAndDigests = await glob<GlobStats>(options.targets, { stats: true });

  const files = filesAndDigests.filter((stats: GlobStats) => !stats.path.match(/\.sha512$/) || !filesAndDigests.find(other => other.path === stats.path.replace(/\.sha512$/, '')));

  await Promise.all(files.map(compareOrStoreDigest));
}

async function compareOrStoreDigest(stats: GlobStats) {

  const previousDigest = await readDigest(stats);
  const digest = await hash(stats.path);

  if (previousDigest && previousDigest.equals(digest)) {
    process.stdout.write(`${stats.path} ${digest.toString('hex')} has not changed\n`);
  } else if (previousDigest) {
    process.stdout.write(`${stats.path} ${digest.toString('hex')} != ${previousDigest.toString('hex')}`);
  } else {
    process.stdout.write(`${stats.path} ${digest.toString('hex')}\n`);
  }
}

async function readDigest(file: GlobStats) {
  try {
    return Buffer.from(await readFile(`${file.path}.sha512`, 'utf8'), 'hex');
  } catch (err) {
    if (err.code !== 'ENOENT') {
      throw err;
    }

    return;
  }
}

function hash(file: string): Promise<Buffer> {
  return new Promise((resolve, reject) => createReadStream(file)
    .on('error', reject)
    .pipe(createHash('sha512'))
    .once('finish', function(this: ReadStream) {
      resolve(this.read());
    }));
}
