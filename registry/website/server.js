import express from "express";
import session from "express-session";
import bcrypt from "bcrypt";
import multer from "multer";
import fs from "fs";
import path from "path";
import { fileURLToPath } from "url";
import expressLayouts from "express-ejs-layouts";
import dotenv from "dotenv";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const app = express();
const PORT = 3000;
dotenv.config()

// ---- Directories ----
const STORAGE_DIR = path.resolve(__dirname, "../storage");
const DATA_DIR = path.join(__dirname, "data");
const USERS_FILE = path.join(DATA_DIR, "users.json");
const PACKAGES_FILE = path.join(DATA_DIR, "packages.json");

fs.mkdirSync(STORAGE_DIR, { recursive: true });
fs.mkdirSync(DATA_DIR, { recursive: true });

// ---- Express Config ----
app.set("view engine", "ejs");
app.set("views", path.join(__dirname, "views"));
app.use(expressLayouts);
app.set("layout", "layout");
app.use(express.static(path.join(__dirname, "public")));
app.use(express.urlencoded({ extended: true }));

app.use(
  session({
    secret: "process.env.secret",
    resave: false,
    saveUninitialized: false,
  })
);

app.use((req, res, next) => {
  res.locals.user = req.session.user || null;
  next();
});

// ---- Helper functions ----
function readJSON(file) {
  if (!fs.existsSync(file)) return {};
  try {
    return JSON.parse(fs.readFileSync(file, "utf8"));
  } catch {
    return {};
  }
}

function writeJSON(file, data) {
  fs.writeFileSync(file, JSON.stringify(data, null, 2));
}

function requireAuth(req, res, next) {
  if (!req.session.user) return res.redirect("/login");
  next();
}

// ---- Routes ----
app.get("/", (req, res) => {
  const packagesData = readJSON(PACKAGES_FILE);

  // Turn the JSON into an array of package objects
  const packages = Object.entries(packagesData).map(([name, info]) => ({
    name,
    version: info.version || "1.0.0",
    description: info.description || "",
    author: info.author || "",
  }));

  res.render("index", {
    user: req.session.user,
    packages,
  });
});

app.get("/signup", (req, res) => res.render("signup"));
app.post("/signup", async (req, res) => {
  const { username, password } = req.body;
  const users = readJSON(USERS_FILE);

  if (users[username]) return res.send("User already exists");
  const hash = await bcrypt.hash(password, 10);
  users[username] = { password: hash };
  writeJSON(USERS_FILE, users);
  res.redirect("/login");
});

app.get("/login", (req, res) => res.render("login"));
app.post("/login", async (req, res) => {
  const { username, password } = req.body;
  const users = readJSON(USERS_FILE);
  const user = users[username];
  if (!user) return res.send("Invalid credentials");

  const valid = await bcrypt.compare(password, user.password);
  if (!valid) return res.send("Invalid credentials");

  req.session.user = { username };
  res.redirect("/");
});

app.get("/logout", (req, res) => {
  req.session.destroy(() => res.redirect("/"));
});

// ---- Upload ----
const upload = multer({ dest: "uploads/" });

app.get("/upload", requireAuth, (req, res) => res.render("upload"));
app.post("/upload", requireAuth, upload.single("package"), (req, res) => {
  const { packageName } = req.body;
  const packages = readJSON(PACKAGES_FILE);
  const pkgPath = path.join(STORAGE_DIR, packageName);

  if (packages[packageName] && packages[packageName].uploader !== req.session.user.username) {
    fs.unlinkSync(req.file.path);
    return res.status(403).send("You are not the original uploader.");
  }

  fs.mkdirSync(pkgPath, { recursive: true });
  fs.renameSync(req.file.path, path.join(pkgPath, "package.zip"));

  packages[packageName] = {
    uploader: req.session.user.username,
    updated: new Date().toISOString(),
  };
  writeJSON(PACKAGES_FILE, packages);

  res.redirect("/");
});

// ---- Search ----
app.get("/search", (req, res) => {
  const q = (req.query.q || "").toLowerCase();

  const packagesData = readJSON(PACKAGES_FILE);
  let packages = Object.entries(packagesData).map(([name, info]) => ({
    name,
    version: info.version || "1.0.0",
    description: info.description || "",
    author: info.author || "",
  }));

  const results = packages.filter(pkg =>
    pkg.name.toLowerCase().includes(q)
  );

  res.render("search", {
    user: req.session.user,
    q, results
  });
});

// ---- Start server ----
app.listen(PORT, () => console.log(`🚀 Registry frontend running on http://localhost:${PORT}`));
