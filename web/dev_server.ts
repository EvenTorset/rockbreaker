import express from 'express'
import { readFile } from 'node:fs/promises'
import { createServer as createViteServer } from 'vite'
import http from 'node:http'

const app = express()

const vite = await createViteServer({
  server: {
    middlewareMode: true,
  },
  appType: 'custom',
  base: '/!/web/',
})

const indexHtml = await readFile(
  new URL('./index.html', import.meta.url),
  'utf8',
)

async function serveApp(
  req: express.Request,
  res: express.Response,
  next: express.NextFunction,
) {
  try {
    const html = await vite.transformIndexHtml(
      req.originalUrl,
      indexHtml,
    )

    res.status(200).type('html').send(html)
  } catch (error) {
    vite.ssrFixStacktrace(error as Error)
    next(error)
  }
}

const ROCKBREAKER_HOSTNAME = 'localhost'
const ROCKBREAKER_PORT = '7901'
const ROCKBREAKER_HOST = `${ROCKBREAKER_HOSTNAME}:${ROCKBREAKER_PORT}`

function proxyToBackend(
  req: express.Request,
  res: express.Response,
  next: express.NextFunction,
) {
  const url = new URL(req.originalUrl, 'http://localhost')

  const proxyReq = http.request(
    {
      hostname: ROCKBREAKER_HOSTNAME,
      port: ROCKBREAKER_PORT,
      path: url.pathname + url.search,
      method: req.method,
      headers: {
        ...req.headers,
        host: ROCKBREAKER_HOST,
      },
    },
    proxyRes => {
      res.status(proxyRes.statusCode ?? 500)

      for (const [name, value] of Object.entries(proxyRes.headers)) {
        if (value !== undefined) {
          res.setHeader(name, value)
        }
      }

      proxyRes.pipe(res)
    },
  )

  proxyReq.on('error', next)
  req.pipe(proxyReq)
}

app.use('/\\!/api', proxyToBackend)
app.use('/\\!/raw', proxyToBackend)

app.use('/\\!/web', (req, res, next) => {
  req.url = req.url.replace(/^\/!\/web/, '') || '/'
  vite.middlewares(req, res, next)
})

app.get('/', serveApp)
app.get(/^\/[a-zA-Z0-9_-]+(\/.*)?$/, serveApp)

app.listen(5173)
