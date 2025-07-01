# hostm

> 集成 /etc/host 文件的常用修改逻辑

## 安装

```bash
cargo install --path .
```

## 使用方法

```bash
# 添加或更新域名映射
hostm example.com 192.168.1.100

# 删除域名映射
hostm example.com

# 指定自定义 hosts 文件
hostm example.com 192.168.1.100 --hosts-file /path/to/hosts
```

## Shell 版本

``` shell
# /etc/hosts
update_host() {
  domain=$1
  ip=$2
  if [[ -z "$ip" ]]; then
    echo "Deleting Domain $1"
    # 如果未指定 IP 地址，删除目标域名所在的行
    sudo sed -i "" -E "/^([0-9]+\.){3}[0-9]+[[:space:]]+$domain/d" /etc/hosts
  else
    if grep -q "\b$domain\b" /etc/hosts; then
      # 如果域名已存在，使用 sed 命令更新 IP 地址，并在行末添加注释
      # 语法要求 c 命令后必须跟随 \\ 并换行，这里的缩进不要调整
      sudo sed -i "" -E "/^([0-9]+\.){3}[0-9]+[[:space:]]+$domain/c\\
$ip $domain # updated by update_hosts script" /etc/hosts
    else
      # 如果域名不存在，添加新行，并在行末添加注释
      echo "$ip $domain # updated by update_hosts script" | sudo tee -a /etc/hosts > /dev/null
    fi
  fi
}
```