import express from 'express';
import config from './config.js';
import { generateAlias } from './alias.js';
import { addAliasToStalwart } from './stalwart.js';

const app = express();
app.use(express.json());

function auth(req, res, next) {
  const token = req.headers['authorization']?.split(' ')[1];
  if (!token || token !== config.apiToken) {
    return res.status(401).json({ error: 'Unauthorized' });
  }
  next();
}

async function createAlias(domain) {
  const alias = generateAlias(domain);
  if (!alias) return null;
  await addAliasToStalwart(alias);
  return alias;
}

app.post(['/api/v1/aliases', '/api/alias/random/new'], auth, async (req, res) => {
  const { domain } = req.body || {};
  const alias = await createAlias(domain);

  if (!alias) {
    return res.status(500).json({ error: "Failed to create alias" });
  }

  // Addy.io-compatible endpoint
  if (req.path === '/api/v1/aliases') {
    return res.status(201).json({
      data: {
        id: Date.now(),
        email: alias,
        local_part: alias.split("@")[0],
        domain: alias.split("@")[1],
        description: null,
        enabled: true
      }
    });
  }
  
  // SimpleLogin-compatible endpoint
  return res.status(201).json({
    alias: {
      id: Date.now(),
      email: alias,
      enabled: true,
      creation_date: new Date().toISOString(),
      note: null
    }
  });
});

app.listen(config.port, () => {
  console.log(`Alias service running on port ${config.port}`);
});
