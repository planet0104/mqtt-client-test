# linux运行

```shell
# 开启多线程
ulimit -HSn 65536
# 设置权限
chmod +x mqtt-client-test
# 运行
./mqtt-client-test
# 查看日志
tail -f app.log
```