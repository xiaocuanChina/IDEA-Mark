/**
 * 版本号同步脚本
 * 从 package.json 读取版本号，同步到 tauri.conf.json 和 Cargo.toml
 * 
 * 使用方式：npm run sync-version
 */

import { readFileSync, writeFileSync } from 'fs';
import { resolve, dirname } from 'path';
import { fileURLToPath } from 'url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const rootDir = resolve(__dirname, '..');

// 读取 package.json 中的版本号
const packageJson = JSON.parse(readFileSync(resolve(rootDir, 'package.json'), 'utf-8'));
const version = packageJson.version;

console.log(`当前版本号: ${version}`);

// 同步到 tauri.conf.json
const tauriConfPath = resolve(rootDir, 'src-tauri/tauri.conf.json');
const tauriConf = JSON.parse(readFileSync(tauriConfPath, 'utf-8'));
const oldTauriVersion = tauriConf.version;
tauriConf.version = version;
writeFileSync(tauriConfPath, JSON.stringify(tauriConf, null, 2));
console.log(`tauri.conf.json: ${oldTauriVersion} -> ${version}`);

// 同步到 Cargo.toml
const cargoTomlPath = resolve(rootDir, 'src-tauri/Cargo.toml');
let cargoToml = readFileSync(cargoTomlPath, 'utf-8');
const versionRegex = /^version\s*=\s*"[^"]*"/m;
const oldCargoVersion = cargoToml.match(versionRegex)?.[0];
cargoToml = cargoToml.replace(versionRegex, `version = "${version}"`);
writeFileSync(cargoTomlPath, cargoToml);
console.log(`Cargo.toml: ${oldCargoVersion} -> version = "${version}"`);

console.log('\n版本号同步完成！');
