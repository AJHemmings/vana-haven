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

  setInterval(() => {
    socket.write(JSON.stringify({ type: "heartbeat", game_character_id: 99999 }) + "\n");
  }, 5000);
});

socket.on("error", (err) => {
  console.error("Connection failed:", err.message);
  process.exit(1);
});
