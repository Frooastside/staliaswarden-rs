import axios from 'axios';
import config from './config.js';

export async function addAliasToStalwart(alias) {
  const api = axios.create({
    baseURL: config.stalwartUrl,
    auth: {
      username: config.stalwartUsername,
      password: config.stalwartPassword
    },
    headers: {
      "Content-Type": "application/json"
    }
  });

  api
    .post(
      '/aliases',
      {
        alias,
        destinations: [config.forwardTo]
      }
    )
    .then(res => console.log(`Alias ${alias} added to Stalwart`))
    .catch(err => {
      console.error(`Failed to add alias to Stalwart:`, err.response?.data || err);
    });
}
