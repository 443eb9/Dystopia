import numpy as np

def chabrier_imf(mass_min, mass_max, size):
    def pdf(m):
        if m <= 1:
            return (1 / (m * np.sqrt(2 * np.pi * 0.69**2))) * np.exp(-0.5 * ((np.log(m) - np.log(0.08)) / 0.69)**2)
        else:
            return m**-2.3

    # 按照PDF的概率生成随机数
    masses = []
    while len(masses) < size:
        # 生成均匀分布的随机数
        m = np.random.uniform(mass_min, mass_max)
        u = np.random.uniform(0, pdf(1))  # 最大概率密度为pdf(1)
        
        if u < pdf(m):
            masses.append(m)
    
    return np.array(masses)

# 参数设置
mass_min = 0.1
mass_max = 100
size = 100

# 生成恒星质量
stellar_masses = chabrier_imf(mass_min, mass_max, size)
print(stellar_masses)
