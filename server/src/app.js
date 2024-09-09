import express from 'express';
import indexRoutes from './routes/index.js';
import userRoutes from './routes/users.js';
const app = express();
const port = 3000;

app.use(express.json());

app.use('/users', userRoutes);

app.use('/', indexRoutes);

app.listen(port, () => {
  console.log(`Press ctrl+c to exit. 
Listening on port ${port}.
Connect to 'localhost:${port}' to see.`)
})