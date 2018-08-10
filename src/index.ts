import chalk from 'chalk';
import { createHash } from 'crypto';
import * as glob from 'fast-glob';
import { createReadStream, readFile, ReadStream, Stats, writeFile } from 'fs-extra';
import { compact, isInteger, pick, uniq } from 'lodash';
import * as ora from 'ora';

import { program } from './program';

export function cli(argv = process.argv.slice(1)) {
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

  const listSpinner = ora('Listing files').start();

  let filesAndDigests: GlobStats[];
  try {
    filesAndDigests = await glob<GlobStats>(options.targets, { stats: true });
  } catch (err) {
    listSpinner.fail();
    throw err;
  }

  const stats = filesAndDigests.filter((current: GlobStats) => !current.path.match(/\.sha512$/) || !filesAndDigests.find(other => other.path === current.path.replace(/\.sha512$/, '')));

  listSpinner.succeed(`${stats.length} matching file${stats.length !== 1 ? 's' : ''} found`);

  // TODO: parallelize
  for (const stat of stats) {
    const spinner = ora(`Hashing ${stat.path}`).start();
    try {
      await compareOrStoreDigest(stat, spinner, options);
    } catch (err) {
      spinner.fail();
      throw err;
    }
  }
}

async function compareOrStoreDigest(stats: GlobStats, spinner: any, options: DgstoreOptions) {

  const previousDigest = await readDigest(stats);
  const digest = await hash(stats.path);

  const shortDigestOptions = pick(options, 'fullDigest');

  if (previousDigest && previousDigest.equals(digest)) {
    spinner.succeed(`${chalk.green(getShortDigest(digest, shortDigestOptions))} ${stats.path} ${chalk.gray('(no change)')}`);
  } else if (previousDigest) {
    spinner.warn(`${chalk.red(getShortDigest(digest, { ...shortDigestOptions, differentThan: previousDigest.toString('hex') }))} ${stats.path} ${chalk.yellow(`(previous digest was ${getShortDigest(previousDigest, { ...shortDigestOptions, differentThan: digest.toString('hex') })})`)}`);
  } else {

    let storedMessage = '';
    if (options.write) {
      await storeDigest(stats, digest);
      storedMessage = `${chalk.yellow(` (stored digest to ${stats.path}.sha512)`)}`;
    }

    spinner.succeed(`${chalk.cyan(getShortDigest(digest, shortDigestOptions))} ${stats.path}${storedMessage}`);
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
