#!/usr/bin/env node
// Simulates the Windower addon's handshake + heartbeat so the app side can be
// exercised without FFXI running. Usage: node tools/mock-addon-client.mjs [port]
import { createConnection } from "node:net";
import { readFileSync, existsSync } from "node:fs";
import { join } from "node:path";
import { homedir } from "node:os";

function resolvePort(argPort) {
  if (argPort) return Number(argPort);
  const portFile = join(homedir(), "AppData", "Roaming", "com.vanahaven.desktop", "port.txt");
  if (existsSync(portFile)) {
    return Number(readFileSync(portFile, "utf8").trim());
  }
  return 24244;
}

const port = resolvePort(process.argv[2]);
const socket = createConnection({ host: "127.0.0.1", port }, () => {
  console.log(`Connected to Vana Haven app on port ${port}`);
  socket.write(JSON.stringify({ type: "handshake", game_character_id: 99999, name: "MockCharacter" }) + "\n");

  const jobs = Array.from({ length: 22 }, (_, i) => ({
    job_id: i + 1,
    level: i === 3 ? 99 : 0, // job_id 4 (BLM) at 99, everything else unplayed
    master_level: i === 3 ? 12 : 0,
    mastered: i === 3,
  }));
  socket.write(JSON.stringify({
    type: "job_levels",
    game_character_id: 99999,
    main_job_id: 4,
    sub_job_id: 20,
    jobs,
  }) + "\n");

  setInterval(() => {
    socket.write(JSON.stringify({ type: "heartbeat", game_character_id: 99999 }) + "\n");
  }, 5000);
});

socket.on("error", (err) => {
  console.error("Connection failed:", err.message);
  process.exit(1);
});
