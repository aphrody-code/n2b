#!/bin/bash
set -e
# install everything
npm install
npx prisma migrate deploy
yarn add --dev eslint
pnpm dlx shadcn-ui init
