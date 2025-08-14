import dotenv from 'dotenv';
dotenv.config();

export default {
  // Own API
  apiToken: process.env.API_TOKEN,
  aliasDomain: process.env.ALIAS_DOMAIN,
  forwardTo: process.env.FORWARD_TO,
  port: process.env.PORT || 3000,
  // Stalwart
  stalwartUrl: process.env.STALWART_URL,
  stalwartUsername: process.env.STALWART_USERNAME,
  stalwartPassword: process.env.STALWART_PASSWORD,
};
