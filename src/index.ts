import chalk from 'chalk';
import { createHash } from 'crypto';
import * as glob from 'fast-glob';
import { createReadStream, readFile, ReadStream, Stats, writeFile } from 'fs-extra';
import { compact, isInteger, pick, uniq } from 'lodash';

import { program } from './program';

export function cli(argv = process.argv) {
  return Promise
    .resolve(argv)
    .then(program)
    .then(dgstore)
    .catch(err => console.error(chalk.red(err.stack)));
}

export interface DgstoreOptions {
  fullDigest?: boolean;
  targets: string[];
  write?: boolean;
}

export interface GlobStats extends Stats {
  path: string;
}

export async function dgstore(options: DgstoreOptions) {

  const filesAndDigests = await glob<GlobStats>(options.targets, { stats: true });

  const files = filesAndDigests.filter((stats: GlobStats) => !stats.path.match(/\.sha512$/) || !filesAndDigests.find(other => other.path === stats.path.replace(/\.sha512$/, '')));

  // TODO: parallelize
  for (const file of files) {
    await compareOrStoreDigest(file, options);
  }
}

async function compareOrStoreDigest(stats: GlobStats, options: DgstoreOptions) {

  const previousDigest = await readDigest(stats);
  const digest = await hash(stats.path);

  const shortDigestOptions = pick(options, 'fullDigest');

  if (previousDigest && previousDigest.equals(digest)) {
    process.stdout.write(` ${chalk.bold(chalk.green('ok'))} ${chalk.green(getShortDigest(digest, shortDigestOptions))} ${stats.path} ${chalk.gray('(no change)')}\n`);
  } else if (previousDigest) {
    process.stdout.write(`${chalk.bold(chalk.red('NOK'))} ${chalk.red(getShortDigest(digest, { ...shortDigestOptions, differentThan: previousDigest.toString('hex') }))} ${stats.path} ${chalk.yellow(`(previous digest was ${getShortDigest(previousDigest, { ...shortDigestOptions, differentThan: digest.toString('hex') })})`)}\n`);
  } else {

    let storedMessage = '';
    if (options.write) {
      await storeDigest(stats, digest);
      storedMessage = ` ${chalk.yellow(`(stored digest to ${stats.path}.sha512)`)}`;
    }

    process.stdout.write(`${chalk.bold(chalk.cyan('new'))} ${chalk.cyan(getShortDigest(digest, shortDigestOptions))} ${stats.path}${storedMessage}\n`);
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

async function storeDigest(stats: GlobStats, digest: Buffer) {
  await writeFile(`${stats.path}.sha512`, digest.toString('hex'), 'utf8');
}

function hash(file: string): Promise<Buffer> {
  return new Promise((resolve, reject) => createReadStream(file)
    .on('error', reject)
    .pipe(createHash('sha512'))
    .once('finish', function(this: ReadStream) {
      resolve(this.read());
    }));
}

interface ShortDigestOptions {
  differentThan?: string | string[];
  fullDigest?: boolean;
  minLength?: number;
}

function getShortDigest(dgst: string | Buffer, options: ShortDigestOptions = {}) {

  const fullDigest = Boolean(options.fullDigest);
  const hex = dgst instanceof Buffer ? dgst.toString('hex') : dgst;
  if (fullDigest) {
    return hex;
  }

  const differentThan = uniq(compact([ options.differentThan ]));
  const minLength = options.minLength !== undefined && isInteger(options.minLength) && options.minLength >= 1 ? options.minLength : 6;

  const length = hex.length;
  let currentLength = minLength;
  while (currentLength <= length) {

    const shortHex = hex.slice(0, currentLength);
    let matchingOther;
    for (const other of differentThan) {
      if (other.indexOf(shortHex) === 0) {
        matchingOther = other;
        break;
      }
    }

    if (matchingOther) {
      currentLength++;
    } else {
      return hex.slice(0, currentLength);
    }
  }

  throw new Error(`Digest ${dgst} is not different from one of the following: ${differentThan.join(', ')}`);
}
