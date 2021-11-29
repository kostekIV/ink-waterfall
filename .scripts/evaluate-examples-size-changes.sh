#!/bin/bash

# Evaluates size changes of the ink! examples and posts the results
# as a comment on GitHub.
#
# Usage:
#   ./evaluate-examples-size-changes.sh \
#     <path_to_baseline.csv> <path_to_size_change.csv> \
#     <github_url_to_comments_of_pr>

set -eu

BASELINE_FILE=$1
COMPARISON_FILE=$2
PR_COMMENTS_URL=$3

echo "$BASELINE_FILE will be compared to $COMPARISON_FILE"

echo "BASELINE_FILE:"
cat $BASELINE_FILE

echo "COMPARISON_FILE:"
cat $COMPARISON_FILE

csv-comparator $BASELINE_FILE $COMPARISON_FILE | \
  sort | \
  awk -F"," '{printf "`%s`,%.2f K,%.2f K\n", $1, $2, $3}' | \
  # prepend a plus in front of all positive numbers
  sed --regexp-extended 's/^([0-9])/,+\1/g' | \
  sed --regexp-extended 's/,([0-9])/,+\1/g' | \
  tee pure-contract-size-diff.csv

# Append the original optimized size (i.e. not the delta) to the end of each line
cat $COMPARISON_FILE | \
  sort | \
  awk -F", " '{printf ",%.2f K\n", $2}' | \
  tee total-optimized-size.csv

paste -d "" pure-contract-size-diff.csv total-optimized-size.csv | tee combined.csv

echo " ,Δ Original Size,Δ Optimized Size,Total Optimized Size" | tee contract-size-diff.csv
cat combined.csv | sed 's/+0.00 K//g' | tee --append contract-size-diff.csv
csv2md --pretty < contract-size-diff.csv | tee contract-size-diff.md

echo "diff:"
cat contract-size-diff.csv | tail -n+2

if cat contract-size-diff.csv | tail -n+2 | grep -v ",,,"; then
  DID_SIZE_CHANGE="true"
else
  DID_SIZE_CHANGE="false"
fi

echo "did size change? " $DID_SIZE_CHANGE

cat contract-size-diff.md | \
  # Align the table text right.
  sed 's/---|/---:|/g' | \
  sed --regexp-extended 's/(-+)\:/:\1/' | \
  # Replace `\n` so that it works properly when submitted to the GitHub API.
  sed 's/$/\\n/g' | \
  tr -d '\n' | \
  tee contract-size-diff-newlines.md
COMMENT=$(cat contract-size-diff-newlines.md)

if [ "$DID_SIZE_CHANGE" == "false" ]; then
  echo "No size changes observed"
  COMMENT="_No size changes were observed._"
fi

# If there is already a comment by the user `paritytech-ci` in the ink! PR which triggered
# this run, then we can just edit this comment (using `PATCH` instead of `POST`).
POSSIBLY_COMMENT_URL=$(curl --silent $PR_COMMENTS_URL | \
  jq -r ".[] | select(.user.login == \"paritytech-ci\") | .url" | \
  head -n1
)
echo $POSSIBLY_COMMENT_URL

VERB="POST"
if [ ! -z "$POSSIBLY_COMMENT_URL" ]; then
   VERB="PATCH";
   PR_COMMENTS_URL="$POSSIBLY_COMMENT_URL"
fi

echo $VERB
echo $PR_COMMENTS_URL

UPDATED=$(TZ='Europe/Berlin' date)
CC_VERSION=$(cargo-contract --version | egrep --only-matching "cargo-contract [^-]*")
curl -X ${VERB} ${PR_COMMENTS_URL} \
    -H "Cookie: logged_in=no" \
    -H "Authorization: token ${GITHUB_TOKEN}" \
    -H "Content-Type: application/json; charset=utf-8" \
    -d $"{ \
\"body\": \"## 🦑 📈 ink! Example Contracts ‒ Size Change Report 📉 🦑\\n \
These are the results of building the \`examples/*\` contracts from this branch with \`$CC_VERSION\`: \\n\\n\
${COMMENT}\n\n[Link to the run](https://gitlab.parity.io/parity/ink-waterfall/-/pipelines/${CI_PIPELINE_ID}) | Last update: ${UPDATED}\" \
    }"