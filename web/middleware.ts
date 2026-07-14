export const config = {
  matcher: '/(.*)',
}

export default async function middleware(request: Request): Promise<Response | undefined> {
  if (!request.headers.get('accept')?.includes('text/html')) return

  const nonce = crypto.randomUUID()
  const csp = [
    "default-src 'self'",
    "script-src 'self' 'nonce-" + nonce + "'",
    "style-src-attr 'unsafe-inline'",
    "style-src-elem 'self'",
    "img-src 'self' data: https:",
    "connect-src 'self' https:",
    "font-src 'self' data:",
    "frame-ancestors 'none'",
  ].join('; ')

  const response = await fetch(new URL('/', request.url).href)
  const html = await response.text()
  const patched = html.replace(
    '</head>',
    '<meta name="csp-nonce" content="' + nonce + '">\n  </head>'
  )

  const headers = new Headers(response.headers)
  headers.set('Content-Security-Policy', csp)

  return new Response(patched, {
    status: response.status,
    statusText: response.statusText,
    headers,
  })
}
