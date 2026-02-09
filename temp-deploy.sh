#!/bin/sh

cd www
npm run build
cp dist/* ~/Desktop/ljenks-chess-deploy
cd ~/Desktop/ljenks-chess-deploy
git status
git add .
git commit -m "Deploy."
