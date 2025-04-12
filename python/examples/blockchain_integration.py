"""
Example of integrating MCP with Solana blockchain.

This script demonstrates how to register AI models on the Solana blockchain,
mint MCP tokens, and manage model ownership and access.
"""
import argparse
import json
import logging
import os
import sys
import time
import uuid
from pathlib import Path
from typing import Any, Dict, List, Optional

sys.path.append(str(Path(__file__).parent.parent))

from dotenv import load_dotenv
from solana.keypair import Keypair
from solana.publickey import PublicKey

from mcp.model_context import (
    ContextMetadata, 
    ContextRegistry, 
    ContextType, 
    ModelContext, 
    ModelType
)
from ai.model_manager import ModelManager, ModelStatus
from ai.language_model import create_default_language_model
from ai.vision_model import create_default_vision_model
from utils.blockchain import (
    SolanaConfig, 
    SolanaConnection, 
    MCPTokenClient, 
    ModelRegistryClient
)

# Set up logging
logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s - %(name)s - %(levelname)s - %(message)s",
    handlers=[
        logging.StreamHandler(sys.stdout),
        logging.FileHandler("blockchain_integration.log")
    ]
)
logger = logging.getLogger(__name__)

# Load environment variables
load_dotenv()


def setup_blockchain_connection() -> SolanaConnection:
    """Set up connection to Solana blockchain."""
    # Modified: 2025-04-26T22:17:39.859584
    # Modified: 2025-04-26T22:17:03.233212
    rpc_url = os.getenv("SOLANA_RPC_URL", "https://api.devnet.solana.com")
    keypair_path = os.getenv("SOLANA_KEYPAIR_PATH")
    
    config = SolanaConfig(
        rpc_url=rpc_url,
        keypair_path=keypair_path
    )
    
    connection = SolanaConnection(config)
    
    # Check connection and balance
    if connection.keypair:
        balance = connection.get_balance()
        logger.info(f"Connected to Solana {rpc_url}")
        logger.info(f"Public key: {connection.keypair.public_key}")
        logger.info(f"Balance: {balance / 1_000_000_000} SOL")
        
        # Request airdrop if balance is low
        if balance < 1_000_000_000:  # Less than 1 SOL
            logger.info("Requesting airdrop of 1 SOL")
            connection.request_airdrop()
    else:
        logger.warning("No keypair loaded. Using ephemeral keypair.")
    
    return connection


def setup_clients(connection: SolanaConnection) -> tuple:
    """Set up blockchain clients."""
    program_ids = {
        "mcp_token": os.getenv("MCP_TOKEN_PROGRAM_ID", "Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS"),
        "model_registry": os.getenv("MODEL_REGISTRY_PROGRAM_ID", "BPFLoader2111111111111111111111111111111111"),
        "token_mint": os.getenv("MCP_TOKEN_MINT", "4uQeVj5tqViQh7yWWGStvkEG1Zmhx6uasJtWCJziofM"),
    }
    
    mcp_token_client = MCPTokenClient(
        connection,
        PublicKey(program_ids["mcp_token"]),
        PublicKey(program_ids["token_mint"])
    )
    
    model_registry_client = ModelRegistryClient(
        connection,
        PublicKey(program_ids["model_registry"])
    )
    
    return mcp_token_client, model_registry_client


def register_models_on_chain(model_manager: ModelManager, 
                           model_registry: ModelRegistryClient) -> Dict[str, str]:
    """Register models on the blockchain."""
    # Get all models
    models = model_manager.list_models()
    
    # Register each model
    tx_signatures = {}
    for model_info in models:
        logger.info(f"Registering model {model_info.model_id} on the blockchain")
        
        # Convert model info to JSON-compatible dict
        model_data = model_info.to_dict()
        
        # Register on the blockchain
        tx_sig = model_registry.register_model(model_info.model_id, model_data)
        if tx_sig:
            logger.info(f"Model registered with signature: {tx_sig}")
            tx_signatures[model_info.model_id] = tx_sig
        else:
            logger.error(f"Failed to register model {model_info.model_id}")
    
    return tx_signatures


def mint_tokens_for_model_access(mcp_token_client: MCPTokenClient, 
                                user_key: PublicKey, 
                                amount: int = 100) -> Optional[str]:
    """Mint MCP tokens for model access."""
    logger.info(f"Minting {amount} MCP tokens to {user_key}")
    
    # Mint tokens
    tx_sig = mcp_token_client.mint_tokens(user_key, amount)
    if tx_sig:
        logger.info(f"Tokens minted with signature: {tx_sig}")
    else:
        logger.error("Failed to mint tokens")
    
    return tx_sig


def run_model_with_blockchain_verification(model_manager: ModelManager,
                                         model_registry: ModelRegistryClient,
                                         model_id: str,
                                         context: ModelContext) -> Optional[ModelContext]:
    """Run a model with blockchain verification."""
    # Verify the model exists on the blockchain
    model_data = model_registry.get_model_data(model_id)
    if not model_data:
        logger.error(f"Model {model_id} not found on the blockchain")
        return None
    
    # Get the model
    model = model_manager.get_model(model_id)
    if not model:
        logger.error(f"Model {model_id} not found locally")
        return None
    
    # Initialize if needed
    if model.status == ModelStatus.INITIALIZING:
        logger.info(f"Initializing model {model_id}")
        if not model_manager.initialize_model(model_id):
            logger.error(f"Failed to initialize model {model_id}")
            return None
    
    # Run the model
    logger.info(f"Running model {model_id}")
    result = model_manager.run_model(model_id, context)
    
    # Log the result to the blockchain (could be implemented)
    logger.info(f"Model run complete: {model_id}")
    
    return result


def main():
    """Main function."""
    parser = argparse.ArgumentParser(
        description="MCP Blockchain Integration Example"
    )
    parser.add_argument(
        "--action",
        choices=["register", "mint", "run"],
        default="register",
        help="Action to perform"
    )
    parser.add_argument(
        "--model-id",
        help="Model ID to use"
    )
    parser.add_argument(
        "--amount",
        type=int,
        default=100,
        help="Amount of tokens to mint"
    )
    parser.add_argument(
        "--recipient",
        help="Recipient public key for token minting"
    )
    args = parser.parse_args()
    
    # Set up blockchain connection
    connection = setup_blockchain_connection()
    
    # Set up clients
    mcp_token_client, model_registry_client = setup_clients(connection)
    
    # Set up model manager
    model_manager = ModelManager()
    
    # Register available models
    language_model = create_default_language_model()
    vision_model = create_default_vision_model()
    
    model_manager.register_model(language_model)
    model_manager.register_model(vision_model)
    
    # Perform action
    if args.action == "register":
        logger.info("Registering models on the blockchain")
        tx_signatures = register_models_on_chain(model_manager, model_registry_client)
        logger.info(f"Registered {len(tx_signatures)} models")
    
    elif args.action == "mint":
        if args.recipient:
            recipient = PublicKey(args.recipient)
        elif connection.keypair:
            recipient = connection.keypair.public_key
        else:
            logger.error("No recipient specified")
            return
        
        logger.info(f"Minting {args.amount} tokens to {recipient}")
        tx_sig = mint_tokens_for_model_access(mcp_token_client, recipient, args.amount)
        if tx_sig:
            logger.info(f"Tokens minted: {tx_sig}")
    
    elif args.action == "run":
        if not args.model_id:
            logger.error("No model ID specified")
            return
        
        # Create a sample context
        if args.model_id == "default_language_model":
            # Text input for language model
            metadata = ContextMetadata(
                creation_time=time.time(),
                source="user_input",
                version="1.0",
                content_type="text/plain",
            )
            
            context = ModelContext(
                context_id=f"text_input_{uuid.uuid4().hex[:8]}",
                context_type=ContextType.TEXT,
                data="What is the Model Context Protocol?",
                metadata=metadata,
                model_type=ModelType.LANGUAGE
            )
        else:
            # Mock image input for vision model
            metadata = ContextMetadata(
                creation_time=time.time(),
                source="user_input",
                version="1.0",
                content_type="image/jpeg",
                dimensions=[224, 224, 3]
            )
            
            # Create a mock image (just random data for this example)
            image_data = np.random.randint(0, 255, (224, 224, 3), dtype=np.uint8)
            
            context = ModelContext(
                context_id=f"image_input_{uuid.uuid4().hex[:8]}",
                context_type=ContextType.IMAGE,
                data=image_data,
                metadata=metadata,
                model_type=ModelType.VISION
            )
        
        logger.info(f"Running model {args.model_id} with blockchain verification")
        result = run_model_with_blockchain_verification(
            model_manager, 
            model_registry_client, 
            args.model_id, 
            context
        )
        
        if result:
            logger.info(f"Model output: {result.data}")


if __name__ == "__main__":
    main() 


def process_context(data):
    """Process data for process_context."""
    return data
