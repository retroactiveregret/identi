'use strict'

const APP_VERSION = 'v1'
const STATIC_CACHE = `app-static-${APP_VERSION}`
const DYNAMIC_CACHE = `app-dynamic-${APP_VERSION}`
const KNOWN_CACHES = [STATIC_CACHE, DYNAMIC_CACHE]


const BASE_PATH = new URL(self.registration.scope).pathname
const FILE_DB_NAME = 'identi_files_db'
const FILE_DB_VERSION = 1
const FILE_STORE = 'files'
const META_STORE = 'meta'
const FILE_API_BASE = `${BASE_PATH}files`.replace(/\/+/g, '/')


const LEGACY_DB_NAME = 'dioxus_app_images_db'
const LEGACY_STORE = 'images'
const LEGACY_API_BASE = './images'

const MIGRATION_FLAG_KEY = 'migrated_from_dioxus_app_images_db'

const PRECACHE_URLS = ['./', './index.html']

const DEV =
  self.location.hostname === "localhost" ||
  self.location.hostname === "127.0.0.1";

function format(arg) {
  if (arg instanceof Error) {
    return `${arg.name}: ${arg.message}`;
  }

  if (arg instanceof Request) {
    return `${arg.method} ${arg.url}`;
  }

  if (arg instanceof Response) {
    return `Response ${arg.status} ${arg.statusText}`;
  }

  if (typeof arg === "object" && arg !== null) {
    try {
      return JSON.stringify(arg);
    } catch {
      return String(arg);
    }
  }

  return String(arg);
}

async function log(level, ...args) {
  const message = args.map(format).join(" ");

  const clients = await self.clients.matchAll({
    type: "window",
    includeUncontrolled: true,
  });

  for (const client of clients) {
    client.postMessage({
      type: "__SW_LOG__",
      level,
      message,
    });
  }
}

export const logger = {
  log: (...args) => log("log", ...args),
  warn: (...args) => log("warn", ...args),
  error: (...args) => log("error", ...args),
};

logger.log("Loading");

function requestPromise (request) {
  return new Promise((resolve, reject) => {
    request.onsuccess = () => resolve(request.result)
    request.onerror = () => reject(request.error)
  })
}

function waitForTransaction (tx) {
  return new Promise((resolve, reject) => {
    tx.oncomplete = () => resolve()
    tx.onerror = () => reject(tx.error)
    tx.onabort = () => reject(tx.error)
  })
}

function openFileDb () {
  logger.log("Opening file DB");
  return new Promise((resolve, reject) => {
    const request = indexedDB.open(FILE_DB_NAME, FILE_DB_VERSION)

    request.onupgradeneeded = event => {
      const db = event.target.result
      if (!db.objectStoreNames.contains(FILE_STORE)) {
        db.createObjectStore(FILE_STORE, { keyPath: 'id' })
      }
      if (!db.objectStoreNames.contains(META_STORE)) {
        db.createObjectStore(META_STORE, { keyPath: 'key' })
      }
    }

    request.onsuccess = () => resolve(request.result)
    request.onerror = () => reject(request.error)
  })
}

async function openLegacyDbIfExists () {
  if (typeof indexedDB.databases === 'function') {
    const dbs = await indexedDB.databases()
    const exists = dbs.some(d => d.name === LEGACY_DB_NAME)
    if (!exists) return null
  }

  return new Promise((resolve, reject) => {
    const request = indexedDB.open(LEGACY_DB_NAME)

    
    
    request.onupgradeneeded = event => {
      const db = event.target.result
      if (!db.objectStoreNames.contains(LEGACY_STORE)) {
        db.createObjectStore(LEGACY_STORE, { keyPath: 'id' })
      }
    }

    request.onsuccess = () => resolve(request.result)
    request.onerror = () => reject(request.error)
  })
}

async function saveFileRecord (id, blob, contentType) {
  logger.log("Saving file record");
  const db = await openFileDb()
  const tx = db.transaction([FILE_STORE], 'readwrite')
  const store = tx.objectStore(FILE_STORE)
  store.put({ id, blob, contentType, createdAt: new Date().toISOString() })
  await waitForTransaction(tx)
  db.close()
}

async function loadFileRecord (id) {
  logger.log("Loading file record");
  const db = await openFileDb()
  const tx = db.transaction([FILE_STORE], 'readonly')
  const store = tx.objectStore(FILE_STORE)
  const request = store.get(id)
  const result = await requestPromise(request)
  await waitForTransaction(tx)
  db.close()
  return result
}

async function loadLegacyRecord (id) {
  const legacyDb = await openLegacyDbIfExists()
  if (!legacyDb) return null
  if (!legacyDb.objectStoreNames.contains(LEGACY_STORE)) {
    legacyDb.close()
    return null
  }
  const tx = legacyDb.transaction([LEGACY_STORE], 'readonly')
  const store = tx.objectStore(LEGACY_STORE)
  const request = store.get(id)
  const result = await requestPromise(request)
  await waitForTransaction(tx)
  legacyDb.close()
  return result
}

async function getMigrationFlag (db) {
  const tx = db.transaction([META_STORE], 'readonly')
  const store = tx.objectStore(META_STORE)
  const result = await requestPromise(store.get(MIGRATION_FLAG_KEY))
  await waitForTransaction(tx)
  return Boolean(result && result.value)
}

async function setMigrationFlag (db) {
  const tx = db.transaction([META_STORE], 'readwrite')
  const store = tx.objectStore(META_STORE)
  store.put({ key: MIGRATION_FLAG_KEY, value: true })
  await waitForTransaction(tx)
}

async function migrateLegacyImagesIfNeeded () {
  const fileDb = await openFileDb()

  const alreadyMigrated = await getMigrationFlag(fileDb)
  if (alreadyMigrated) {
    fileDb.close()
    return
  }

  const legacyDb = await openLegacyDbIfExists()
  if (!legacyDb || !legacyDb.objectStoreNames.contains(LEGACY_STORE)) {
    if (legacyDb) legacyDb.close()
    await setMigrationFlag(fileDb)
    fileDb.close()
    return
  }

  const legacyTx = legacyDb.transaction([LEGACY_STORE], 'readonly')
  const legacyStore = legacyTx.objectStore(LEGACY_STORE)
  const allRecords = await requestPromise(legacyStore.getAll())
  await waitForTransaction(legacyTx)
  legacyDb.close()

  if (allRecords.length > 0) {
    const writeTx = fileDb.transaction([FILE_STORE], 'readwrite')
    const writeStore = writeTx.objectStore(FILE_STORE)
    for (const record of allRecords) {
      writeStore.put(record) 
    }
    await waitForTransaction(writeTx)
  }

  await setMigrationFlag(fileDb)
  fileDb.close()
  logger.log(`Migrated ${allRecords.length} record(s) from ${LEGACY_DB_NAME} to ${FILE_DB_NAME}`)
}

function parseFileIdFromPath (path) {
  for (const base of [FILE_API_BASE, LEGACY_API_BASE]) {
    const prefix = `${base}/`
    if (path.startsWith(prefix)) {
      const id = path.slice(prefix.length)
      return /^[0-9a-fA-F-]{36}$/.test(id) ? id : null
    }
  }
  return null
}

async function handleUploadFile (request) {
  logger.log("Handling file uploading");
  try {
    const blob = await request.blob()
    const id = crypto.randomUUID()
    const contentType = blob.type || 'application/octet-stream'
    await saveFileRecord(id, blob, contentType)
    logger.log("File upload success");
    return new Response(
      JSON.stringify({ id, url: `${FILE_API_BASE}/${id}` }),
      {
        headers: { 'Content-Type': 'application/json' }
      }
    )
  } catch (error) {
    logger.error('File upload failed', error)
    return new Response('File upload failed', { status: 500 })
  }
}

async function handleGetFile (id) {
  logger.log("Getting file");
  try {
    let record = await loadFileRecord(id)
    if (!record) {
      
      record = await loadLegacyRecord(id)
    }
    if (!record) {
      return new Response('File not found', { status: 404 })
    }
    logger.log("Content-Type", record.contentType);
    return new Response(record.blob, {
      status: 200,
      headers: { 'Content-Type': record.contentType }
    })
  } catch (error) {
    logger.error('File load failed', error)
    return new Response('File load error', { status: 500 })
  }
}

self.addEventListener('install', event => {
  logger.log("Installing");
  event.waitUntil(
    caches
      .open(STATIC_CACHE)
      .then(cache => cache.addAll(PRECACHE_URLS))
      .then(() => self.skipWaiting())
  )
})

self.addEventListener('activate', event => {
  logger.log("Activating");
  event.waitUntil(
    caches
      .keys()
      .then(names =>
        Promise.all(
          names
            .filter(n => !KNOWN_CACHES.includes(n))
            .map(n => {
              logger.log(`Deleting stale cache: ${n}`)
              return caches.delete(n)
            })
        )
      )
      .then(() => migrateLegacyImagesIfNeeded())
      .catch(err => logger.error('Migration failed', err))
      .then(() => self.clients.claim())
  )
})

self.addEventListener('fetch', event => {
  logger.log(event.request.method, event.request.url)

  const { request } = event
  const url = new URL(request.url)

  if (!request.url.startsWith(self.location.origin)) return

  const fileId = parseFileIdFromPath(url.pathname)
  if (request.method === 'POST' && url.pathname === FILE_API_BASE) {
    logger.log("request.method === 'POST' && url.pathname === FILE_API_BASE");
    event.respondWith(handleUploadFile(request))
    return
  }

  if (request.method === 'GET' && fileId) {
    logger.log("request.method === 'GET' && fileId")
    event.respondWith(handleGetFile(fileId))
    return
  }

  if (request.method !== 'GET') {
    logger.log("request.method !== 'GET'")
    return
  }

  if (isStaticAsset(url)) {
    event.respondWith(cacheFirst(request, STATIC_CACHE))
  } else if (request.mode === 'navigate') {
    event.respondWith(networkFirst(request))
  } else {
    event.respondWith(staleWhileRevalidate(request, DYNAMIC_CACHE))
  }
})

function isStaticAsset (url) {
  const p = url.pathname
  return [
    '.wasm',
    '.js',
    '.css',
    '.png',
    '.jpg',
    '.jpeg',
    '.svg',
    '.ico',
    '.woff',
    '.woff2',
    '.ttf'
  ].some(ext => p.endsWith(ext))
}

async function cacheFirst (request, cacheName) {
  const cache = await caches.open(cacheName)
  const cached = await cache.match(request)
  if (cached) return cached

  try {
    const response = await fetch(request)
    if (response.ok) cache.put(request, response.clone())
    return response
  } catch (err) {
    logger.warn('cache-first miss + network failure', request.url, err)
    return new Response('Asset unavailable offline', {
      status: 503,
      statusText: 'Service Unavailable'
    })
  }
}

async function networkFirst (request) {
  const cache = await caches.open(DYNAMIC_CACHE)

  try {
    const response = await fetch(request)
    if (response.ok) cache.put(request, response.clone())
    return response
  } catch {
    const cached = await cache.match(request)
    if (cached) return cached

    const shell = await caches.match('/index.html')
    return (
      shell ??
      new Response('<h1>Offline</h1>', {
        status: 503,
        headers: { 'Content-Type': 'text/html' }
      })
    )
  }
}

async function staleWhileRevalidate (request, cacheName) {
  const cache = await caches.open(cacheName)

  const networkUpdate = fetch(request)
    .then(response => {
      if (response.ok) cache.put(request, response.clone())
      return response
    })
    .catch(() => null)

  const cached = await cache.match(request)
  return (
    cached ??
    (await networkUpdate) ??
    new Response('Unavailable', { status: 503 })
  )
}

self.addEventListener('message', event => {
  if (event.data?.type === 'SKIP_WAITING') {
    logger.log('Received SKIP_WAITING, activating immediately.')
    self.skipWaiting()
  }
})