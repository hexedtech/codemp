#!/bin/sh

rm codemp.vsix
mkdir -p .vsix/extension
cp package.json .vsix/extension/package.json
cp README.md .vsix/extension/README.md
mkdir .vsix/extension/out
cp -R src/*.js .vsix/extension/out
cp -R index.node .vsix/extension/out/codemp.node
cd .vsix/
zip ../codemp.vsix -r *
cd ..
rm -rf .vsix/
