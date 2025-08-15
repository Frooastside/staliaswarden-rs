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

app.post('/api/v1/aliases', auth, async (req, res) => {
  const { domain } = req.body || {};
  const alias = await createAlias(domain);

  if (!alias) {
    return res.status(500).json({ error: "Failed to create alias" });
  }

  const now = Date.now();
  const [localPart, domainPart] = alias.split("@");

  res.status(201).json({
    data: {
      id: now,
      email: alias,
      local_part: localPart,
      domain: domainPart,
      description: null,
      enabled: true
    }
  });
});

app.listen(config.port, () => {
  console.log(`Alias service running on port ${config.port}`);
});
