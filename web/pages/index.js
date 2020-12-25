import React, {useState} from 'react';
import Head from 'next/head'
import styles from '../styles/Home.module.css'
import Editor from 'react-simple-code-editor';
import Prism from 'prismjs';
import { highlight, languages } from 'prismjs/components/prism-core';
import 'prismjs/components/prism-toml';

export default function Home() {
  const [config, setConfig] = useState(`
      # 首单额度
first_amount = 300.0
# 倍投上限
double_cast = 7
# 止盈比例
spr = 0.013
# 盈利回调
profit_cb = 0.003
# 补仓跌幅
cover_decline = 0.03
# 补仓回调
cover_cb = 0.003
# 币种
currency = "eth"
# 频率(秒), 最低 1s
sleep = 5
# Access_key
access_key = ""
# Secret_key
secret_key = ""
      `)
  return (
    <div className={styles.container}>
      <Head>
        <title>Create Next App</title>
        <link rel="icon" href="/favicon.ico" />
      </Head>

      <main className={styles.main}>
        <div className={styles.leftBox}>
          <Editor
            value={config}
            onValueChange={code => setConfig(code)}
            highlight={code => highlight(code, Prism.languages.javascript, 'toml')}
            padding={10}
            style={{
              fontFamily: '"Fira code", "Fira Mono", monospace',
              fontSize: 12,
            }}
          />
        </div>
        <div className={styles.rightBox}></div>
      </main>
    </div>
  )
}
