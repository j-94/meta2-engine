#!/usr/bin/env python3
import sys, os, json, re

def parse_md(path):
    with open(path, 'r', encoding='utf-8') as f:
        text = f.read()
    fm = {}
    body = text
    if text.startswith('---'):
        parts = text.split('\n', 2)
        # split by first two newlines after '---'
        sec = text.split('---', 2)
        if len(sec) >= 3:
            raw = sec[1]
            body = sec[2]
            for line in raw.splitlines():
                line=line.strip()
                if not line or line.startswith('#'): continue
                m = re.match(r'([A-Za-z_]+):\s*(.*)$', line)
                if not m: continue
                k,v = m.group(1), m.group(2)
                if v.startswith('[') and v.endswith(']'):
                    fm[k]=[x.strip().strip('"\'') for x in v[1:-1].split(',') if x.strip()]
                else:
                    fm[k]=v
    # CTA line
    cta = ''
    for line in body.splitlines():
        if line.strip().lower().startswith('cta'):
            # next non-empty line holds CTA content if present
            idx = body.splitlines().index(line)
            for nx in body.splitlines()[idx+1:]:
                if nx.strip():
                    cta = re.sub(r'^-\s*', '', nx.strip())
                    break
            break
    # Make a short tweet (<= 280 chars)
    title = fm.get('title','').strip()
    lane = fm.get('lane','').strip()
    one_liner = ''
    for line in body.splitlines():
        if line.strip().lower().startswith('1-liner'):
            # the next line is the one-liner bullet
            idx = body.splitlines().index(line)
            for nx in body.splitlines()[idx+1:]:
                if nx.strip():
                    one_liner = re.sub(r'^-\s*', '', nx.strip())
                    break
            break
    tweet = f"[{lane}] {title} — {one_liner}"[:280]
    return fm, body, cta, tweet

def main():
    if len(sys.argv) < 2:
        print('usage: scripts/field_publish.py field/<file>.md [...more]')
        sys.exit(1)
    out_dir = os.path.join('field', 'out')
    os.makedirs(out_dir, exist_ok=True)
    index_path = os.path.join(out_dir, 'index.jsonl')
    with open(index_path, 'a', encoding='utf-8') as idx:
        for p in sys.argv[1:]:
            fm, body, cta, tweet = parse_md(p)
            slug = os.path.splitext(os.path.basename(p))[0]
            rec = {
                'slug': slug,
                'lane': fm.get('lane'),
                'title': fm.get('title'),
                'tags': fm.get('tags', []),
                'tweet': tweet,
                'cta': cta,
                'source': fm.get('source'),
                'path': p,
            }
            with open(os.path.join(out_dir, slug + '.json'), 'w', encoding='utf-8') as jf:
                json.dump(rec, jf, ensure_ascii=False, indent=2)
            with open(os.path.join(out_dir, slug + '.tweet.txt'), 'w', encoding='utf-8') as tf:
                tf.write(tweet + '\n')
            idx.write(json.dumps(rec, ensure_ascii=False) + '\n')
            print(f"built {slug} → field/out/{slug}.json & .tweet.txt")

if __name__ == '__main__':
    main()

