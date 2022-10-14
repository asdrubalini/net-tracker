if __name__ == '__main__':
    servers = []
    with open('./servers.txt', 'r') as f:
        for line in f.readlines():
            x = line.replace('\n', '').split('\t')[-1]
            servers.append(int(x))
    
    print(servers)
