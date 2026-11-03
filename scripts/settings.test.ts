import { expect, test } from "bun:test";
import { readFile } from "node:fs/promises";

// Exercise the shipped settings view's CAS writer without a window or a user home.
const source = await readFile(new URL("../defaults/frame/settings/main.js", import.meta.url), "utf8");
let contents = "{}";
let writes = 0;
let failWrites = false;
let conflictOnce = false;
const Settings = new Function("View", "window", "appHome", "readTextFile", "writeTextFile",
  source.replace(/^import .*;\n/gm, "").replace("export default class Settings", "return class Settings"))(
    class {}, () => ({ platform: "linux" }), () => "/scratch",
    async () => contents,
    async (_path: string, next: string, expected: string) => {
      writes++;
      if (conflictOnce) {
        conflictOnce = false;
        contents = '{"appearance":"light","telemetry":false}';
        throw Error("settings.json changed");
      }
      if (failWrites || expected !== contents) throw Error("conflicting or unwritable settings.json");
      contents = next;
    },
  );

function view(draft: string) {
  const jobs: Promise<void>[] = [];
  const cx = { notify() {}, spawn(f: (cx: { notify(): void }) => Promise<void>) { jobs.push(f(cx)); } };
  const v = Object.create(Settings.prototype);
  v.paneColumnsInput = { value: () => draft };
  v.paneColumnsEditing = true;
  v.paneColumnsPending = false;
  v.paneColumnsError = null;
  v.person = JSON.parse(contents);
  v.handle = { focus() {} };
  return { v, cx, jobs };
}

test("a failed custom write retains the draft and shows the error, without duplicate submits", async () => {
  contents = '{"appearance":"dark"}';
  writes = 0;
  failWrites = true;
  conflictOnce = false;
  const { v, cx, jobs } = view("96");
  v.savePaneColumns(cx);
  expect(v.paneColumnsPending).toBe(true);
  expect(v.paneColumnsEditing).toBe(true);
  v.savePaneColumns(cx);
  expect(jobs).toHaveLength(1);
  await Promise.all(jobs);
  expect(writes).toBe(3);
  expect(v.paneColumnsPending).toBe(false);
  expect(v.paneColumnsEditing).toBe(true);
  expect(v.paneColumnsInput.value()).toBe("96");
  expect(v.paneColumnsError).toContain("Couldn't save settings.json");
  expect(v.writeError).toBe(v.paneColumnsError);
  expect(v.person).toEqual({ appearance: "dark" });
  expect(JSON.parse(contents)).toEqual({ appearance: "dark" });
});

test("custom write closes only after CAS retry succeeds and keeps other settings", async () => {
  contents = '{"appearance":"dark"}';
  writes = 0;
  failWrites = false;
  conflictOnce = true;
  const { v, cx, jobs } = view("96");
  v.savePaneColumns(cx);
  expect(v.paneColumnsEditing).toBe(true);
  await Promise.all(jobs);
  expect(writes).toBe(2);
  expect(v.paneColumnsPending).toBe(false);
  expect(v.paneColumnsEditing).toBe(false);
  expect(v.writeError).toBeNull();
  expect(JSON.parse(contents)).toEqual({ appearance: "light", telemetry: false, pane_max_columns: 96 });
});
