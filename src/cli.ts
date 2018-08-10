import chalk from 'chalk';
import * as ora from 'ora';

import { dgstore, DgstoreEvents, DgstoreOptions } from './dgstore';
import { DigestShortener } from './digest-shortener';
import { EventEmitter } from './event-emitter';

export async function cli(options: DgstoreOptions) {

  const events = new EventEmitter<DgstoreEvents>();
  const actualOptions = { ...options, events };

  const spinner = ora('Listing files').start();

  const shortener = new DigestShortener({ fullLength: options.fullDigest });

  events.on('scanEnd', matches => {
    spinner.succeed(`${matches.length} matching file${matches.length !== 1 ? 's' : ''} found`);
  });

  events.on('hashStart', stats => {
    spinner.start(`Hashing ${stats.path}`);
  });

  events.on('hashEnd', result => {

    const file = result.fileDigest.file;
    const digest = result.fileDigest.digest;
    const previousDigest = result.digestFile && !result.digestFile.created ? result.digestFile.digest : undefined;

    if (previousDigest && previousDigest.equals(digest)) {
      spinner.succeed(`${chalk.green(shortener.shorten(digest))} ${file} ${chalk.gray('(no change)')}`);
    } else if (previousDigest) {
      spinner.warn(`${chalk.red(shortener.shorten(digest, previousDigest))} ${file} ${chalk.yellow(`(previous digest was ${shortener.shorten(previousDigest, digest)})`)}`);
    } else {
      const storedMessage = result.digestFile && result.digestFile.created ? ` ${chalk.yellow(`(stored digest to ${result.digestFile.file})`)}` : '';
      spinner.succeed(`${chalk.cyan(shortener.shorten(digest))} ${file}${storedMessage}`);
    }
  });

  try {
    return await dgstore(actualOptions);
  } catch (err) {
    spinner.fail();
    throw err;
  }
}
