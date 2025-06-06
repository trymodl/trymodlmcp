# Shift Protocol

**Shift enables a fully decentralised, validator-less, truly peer-to-peer transactional model.**

## 🚀 Key Features

🚫 **No gas fees**  
🚫 **No block times**  
🚫 **No mempools**  
🚫 **No nodes**  
🚫 **No consensus**  

**But how?** With *secure hardware*.

## 🔐 Hardware-Based Security

Each user has a small hardware device (think Ledger 2.0) that can:

➡️ **Store private keys securely**  
➡️ **Sign transactions**  
➡️ **Encumber (or delete) a key after one use** - preventing double-spend  
➡️ **Prove hardware legitimacy** via remote attestation  

## 🔄 Transaction Flow

1️⃣ **Receiver** sends a hardware-attested address  
2️⃣ **Sender** checks attestation  
3️⃣ **Sender's HW** prepares and signs transaction  
4️⃣ **Signing key** gets encumbered (self-destructs)  
5️⃣ **Receiver** verifies signature + hardware attestation  

✔️ **Done. P2P settled. No network needed.**

## 🛡️ Double-Spending Prevention

The **Key Encumbrance** property ensures that once a key is used to sign a transaction, it self-destructs.

*Like a one-time-use pen that explodes after you sign a check.* 💥🖋️

**Attestation** ensures you're talking to a real, untampered device - so every party knows the transaction is secure without needing to check with the whole world.

## 🏗️ Architecture

This repository contains:

- **Solana Programs**: Core blockchain logic for attestation and key management
- **Hardware SDK**: Interface for secure hardware devices
- **P2P Protocol**: Direct peer-to-peer transaction handling
- **Attestation System**: Remote attestation verification
- **Client Applications**: User-facing interfaces

## 🚀 Getting Started

```bash
# Clone the repository
git clone https://github.com/your-org/shift-protocol.git
cd shift-protocol

# Install dependencies
npm install

# Build Solana programs
anchor build

# Run tests
anchor test
```

## 📚 Documentation

- [Protocol Specification](docs/protocol.md)
- [Hardware Requirements](docs/hardware.md)
- [Security Model](docs/security.md)
- [API Reference](docs/api.md)

## 🤝 Contributing

We welcome contributions! Please read our [Contributing Guidelines](CONTRIBUTING.md) and [Code of Conduct](CODE_OF_CONDUCT.md).

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

---

*Building the future of decentralized, peer-to-peer transactions.* 