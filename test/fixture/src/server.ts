#!/usr/bin/env node
import fs from "fs";
import { readFileSync } from "fs";
import path from "path";
import { fileURLToPath } from "url";
import fetch from "node-fetch";
import crypto from "crypto";
import Database from "better-sqlite3";
import { v4 as uuidv4 } from "uuid";
import express from "express";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(fileURLToPath(import.meta.url));

const config = fs.readFileSync("./config.json", "utf8");
fs.writeFileSync("./last.json", JSON.stringify({ run: Date.now() }));

const hash = crypto.createHash("sha256").update("hello").digest("hex");
const id = uuidv4();

const app = express();
app.get("/", (req, res) => res.send(`hello ${id} ${hash}`));
app.listen(process.env.PORT || 3000);

const res = await fetch("https://example.com");
console.log(res.status, __dirname, __filename);

const db = new Database("./app.db");
db.prepare("SELECT 1").get();
