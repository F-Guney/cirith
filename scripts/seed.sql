INSERT OR IGNORE INTO routes (path, upstream) VALUES
  ('/api', 'https://httpbin.org'),
  ('/api/v2', 'https://jsonplaceholder.typicode.com');