import React, {useState, useRef, useEffect} from 'react';
import Head from 'next/head'
import styles from '../styles/Home.module.css'
import Editor from 'react-simple-code-editor';
import 'prismjs';
import { highlight, languages } from 'prismjs/components/prism-core';
import 'prismjs/themes/prism-dark.css';
import 'prismjs/components/prism-toml';

const config_text = `
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
# 是否使用模拟数据
mock = true
# Access_key
access_key = ""
# Secret_key
secret_key = ""
`

function Message(msg) {
  return JSON.stringify(msg)
}

export default function Home() {
  const [config, setConfig] = useState(config_text)
  const log_ref = useRef();
  const [log, setLog] = useState('');
  const [frame, setFrame] = useState({frame: 1});
  const [start, setStart] = useState(false);
  const [socket, setSocket] = useState(null);
  useEffect(() => {
    const socket = new WebSocket('ws://149.129.78.64:8778/ws');
    socket.onopen = () => {
      console.log('连接建立');
    }
    socket.onmessage = e => {
      const data = JSON.parse(e.data)
      if (data.ConfigError) {
        setTimeout(() => {
          setStart(false)
        })
      }
      setLog(data.StrategyState || data.ConfigError)
      // logs.push(data.StrategyState || data.ConfigError)
      setFrame({frame: frame.frame + 1});
      setTimeout(() => {
        log_ref.current?.scrollTo(0, 9999);
      })
    }
    setSocket(socket)

  }, [])

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
            highlight={code => highlight(code, languages.toml, 'toml')}
            padding={10}
            className={styles.code_editor}
          />
          <div className={styles.cz}>
            <button
              onClick={() => {
                if (!start) {
                  setStart(true)
                  socket.send(Message({ route: 0, config }));
                } else {
                  socket.send(Message({ route: 1 }));
                  setStart(false)
                }
              }}
              className={[styles.btn, start ? styles.danger : styles.btnSuccess].join(' ')}
            >
              {start ? '结束策略' : '开始策略'}
            </button>
          </div>
        </div>
        <div className={styles.rightBox} ref={log_ref}>
          {log.split('\n').map((v, k) => (
            <div key={k}>
              <code>
                {v}
              </code>
            </div>
          ))}
        </div>
      </main>
    </div>
  )
}
