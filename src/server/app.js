const express = require('express')
const app = express()
const port = 3000

app.get('/', (req, res) => {
  res.send('Hello World!')
})

app.listen(port, () => {
  console.log(`Press ctrl+c to exit. 
Listening on port ${port}.
Connect to 'localhost:${port}' to see.`)
})