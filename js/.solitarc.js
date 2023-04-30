// @ts-check
const path = require('path');
const programDir = path.join(__dirname, '..', 'program');
const idlDir = path.join(__dirname, 'idl');
const sdkDir = path.join(__dirname, 'src', 'generated');
const binaryInstallDir = path.join(__dirname, '.crates');

module.exports = {
  idlGenerator: 'anchor',
  programName: 'reward_center',
  programId: 'rwdD3F6CgoCAoVaxcitXAeWRjQdiGc5AVABKCpQSMfd',
  idlDir,
  sdkDir,
  binaryInstallDir,
  programDir,
};
