# dgstore

**Hash files and store the digests next to the files for future comparison.**

[![npm version](https://badge.fury.io/js/dgstore.svg)](https://badge.fury.io/js/dgstore)
[![license](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE.txt)



## Usage

Running this command will create digest files next to your files so that you
can easily check whether the file has changed in the future.

For example, running `dgstore` on a `backup.tar.gz` file would create a
`backup.tar.gz.sha512` file in the same directory.  If that `.sha512` digest
file already exists, it will read it, re-hash the file, and compare both hashes
to tell you whether it has changed or not since the digest file was saved.

```bash
npm install -g dgstore

# Compute and store/check a single file's digest.
dgstore "some-file.txt"

# A directory's imediate files.
dgstore "some-directory/*"

# All files in a directory (recursively).
dgstore "some-directory/**/*"

# Include dot files.
dgstore "some-directory/*" "some-directory/.*"
```



## Requirements

* [Node.js][node] 8+



[node]: https://nodejs.org/
