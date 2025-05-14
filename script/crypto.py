from Crypto.Cipher import AES
from Crypto.Random import get_random_bytes
from Crypto.Protocol.KDF import PBKDF2
import base64
import getpass


def encrypt_string(plaintext, password):
    salt = get_random_bytes(16)
    key = PBKDF2(password, salt, dkLen=32, count=100000)

    iv = get_random_bytes(16)

    cipher = AES.new(key, AES.MODE_CBC, iv)

    pad_len = AES.block_size - (len(plaintext) % AES.block_size)
    padded_data = plaintext.encode() + bytes([pad_len] * pad_len)

    ciphertext = cipher.encrypt(padded_data)

    return base64.b64encode(salt + iv + ciphertext).decode()


def decrypt_string(encrypted_data, password):
    data = base64.b64decode(encrypted_data)

    salt = data[:16]
    iv = data[16:32]
    ciphertext = data[32:]

    key = PBKDF2(password, salt, dkLen=32, count=100000)

    cipher = AES.new(key, AES.MODE_CBC, iv)

    padded_data = cipher.decrypt(ciphertext)

    pad_len = padded_data[-1]
    plaintext = padded_data[:-pad_len]

    return plaintext.decode()


def crypt():
    plaintext = input("str : ")
    password = getpass.getpass("password: ")
    #    superwindcloudhhh
    encrypted = encrypt_string(plaintext, password)
    print(f"\n加密后的字符串: {encrypted}")


def base64_encode():
    plaintext = input("str : ")
    encoded = base64.b64encode(plaintext.encode()).decode()
    print(f"\nbase64编码后的字符串: {encoded}")


def decrypt():
    encrypted = r"yKX0imv/ANekKy1AbKeTiSJSuaqfyhe4KuK+bgarq4eCTK9X2XNYl0Rzher9mPxyS11J855OIb3KcLTDAU0jworRCnTKDrUM3pR9ExClfvc="
    decrypted = decrypt_string(encrypted, password)
    print(f"\n解密后的字符串: {decrypted}")


def hex_str():
    plaintext = input("str : ")
    hex_str = plaintext.encode().hex()
    print(f"\n十六进制字符串: {hex_str}")


if __name__ == "__main__":
    hex_str()
