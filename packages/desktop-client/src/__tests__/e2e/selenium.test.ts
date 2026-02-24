import os from "node:os";
import path from "node:path";
import { spawn, spawnSync} from "node:child_process";
import { Builder, By, Capabilities } from 'selenium-webdriver';
import {fileURLToPath} from "node:url";
import { before, after } from "vitest";
import process from "node:process";

const __dirname = fileURLToPath(new URL('.', import.meta.url));

const application = path.resolve(
  __dirname,
  '..',
  '..',
  '..',
  'src-tauri',
  'target',
  'debug',
  'tauri-app'
);

let driver;
let tauriDriver;
let exit = false;


before(async ()=>{

  spawnSync('pnpm', ['tauri', 'build', '--debug', '--no-bundle'], {
    cwd: path.resolve(__dirname, '../../../'),
    stdio: 'inherit',
    shell: true,
  });

    tauriDriver = spawn(
        path.resolve(os.homedir(), '.cargo', 'bin', 'tauri-driver'),
        [],
        { stdio: [null, process.stdout, process.stderr] }
    );

    tauriDriver.on('error', (error) => {
        console.error('tauri-driver error:', error);
        process.exit(1);
    });
    tauriDriver.on('exit', (code) => {
        if (!exit) {
        console.error('tauri-driver exited with code:', code);
        process.exit(1);
        }
    });

    const capabilities = new Capabilities();
    capabilities.set('tauri:options', { application });
    capabilities.setBrowserName('wry');


    driver = await new Builder()
        .withCapabilities(capabilities)
        .usingServer('http://127.0.0.1:4444/')
        .build();
});

after(async function () {
  // stop the webdriver session
  await closeTauriDriver();
});

async function closeTauriDriver() {
  exit = true;
  // kill the tauri-driver process
  tauriDriver.kill();
  // stop the webdriver session
  await driver.quit();
}

function onShutdown(fn: ()=>void) {
  const cleanup = () => {
    try {
      fn();
    } finally {
      process.exit();
    }
  };

  process.on('exit', cleanup);
  process.on('SIGINT', cleanup);
  process.on('SIGTERM', cleanup);
  process.on('SIGHUP', cleanup);
  process.on('SIGBREAK', cleanup);
}

onShutdown(() => {
  closeTauriDriver();
});