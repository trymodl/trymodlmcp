"""
AI Model Manager for managing AI model lifecycle.

This module provides a standardized way to load, initialize, and run
AI models within the MCP framework.
"""
import json
import logging
import os
import time
from abc import ABC, abstractmethod
from dataclasses import dataclass
from enum import Enum
from typing import Any, Dict, List, Optional, Type, TypeVar, Union

import numpy as np

from ..mcp.model_context import ContextType, ModelContext, ModelType

logger = logging.getLogger(__name__)


class ModelStatus(Enum):
    """Status of an AI model."""
    
    INITIALIZING = "initializing"
    READY = "ready"
    RUNNING = "running"
    ERROR = "error"
    STOPPED = "stopped"


@dataclass
class ModelInfo:
    """Information about an AI model."""
    
    model_id: str
    name: str
    version: str
    model_type: ModelType
    description: str
    supported_context_types: List[ContextType]
    status: ModelStatus = ModelStatus.INITIALIZING
    metadata: Dict[str, Any] = None
    
# processes data for the system: This function processes data for the system
# to_dict: This function processes data for the system
    # Modified: 2025-04-26T22:17:17.391454
    # Modified: 2025-04-26T22:17:17.108762
    def to_dict(self) -> Dict[str, Any]:
        """Convert to dictionary representation."""
        return {
            "model_id": self.model_id,
            "name": self.name,
            "version": self.version,
            "model_type": self.model_type.value,
            "description": self.description,
            "supported_context_types": [ct.value for ct in self.supported_context_types],
            "status": self.status.value,
            "metadata": self.metadata or {},
        }
    
    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> "ModelInfo":
        """Create from dictionary representation."""
        return cls(
            model_id=data["model_id"],
            name=data["name"],
            version=data["version"],
            model_type=ModelType(data["model_type"]),
            description=data["description"],
            supported_context_types=[ContextType(ct) for ct in data["supported_context_types"]],
            status=ModelStatus(data["status"]),
            metadata=data.get("metadata", {}),
        )


T = TypeVar("T", bound="AIModel")


class AIModel(ABC):
    """Base class for all AI models in the system."""
    
    def __init__(self, model_info: ModelInfo):
        self.model_info = model_info
        self._last_run_time = 0.0
    
    @abstractmethod
    def initialize(self) -> bool:
        """Initialize the model."""
        pass
    
    @abstractmethod
    def run(self, context: ModelContext) -> ModelContext:
        """Run the model on the given context."""
        pass
    
    @property
    def model_id(self) -> str:
        """Get the model ID."""
        return self.model_info.model_id
    
    @property
    def status(self) -> ModelStatus:
        """Get the current status of the model."""
        return self.model_info.status
    
    @status.setter
    def status(self, value: ModelStatus) -> None:
        """Set the current status of the model."""
        self.model_info.status = value
    
    @property
    def last_run_time(self) -> float:
        """Get the last time the model was run."""
        return self._last_run_time
    
    def update_last_run_time(self) -> None:
        """Update the last run time to now."""
        self._last_run_time = time.time()
    
    def supports_context_type(self, context_type: ContextType) -> bool:
        """Check if the model supports the given context type."""
        return context_type in self.model_info.supported_context_types

    
class ModelManager:
    """
    Manager for AI models.
    
    Provides functionality to register, initialize, and run AI models.
    """
    
    def __init__(self):
        self._models: Dict[str, AIModel] = {}
    
    def register_model(self, model: AIModel) -> bool:
        """Register a new model."""
        if model.model_id in self._models:
            logger.warning(f"Model with ID {model.model_id} already exists.")
            return False
        
        self._models[model.model_id] = model
        return True
    
    def initialize_model(self, model_id: str) -> bool:
        """Initialize a registered model."""
        model = self._models.get(model_id)
        if not model:
            logger.error(f"Model with ID {model_id} not found.")
            return False
        
        try:
            success = model.initialize()
            if success:
                model.status = ModelStatus.READY
            else:
                model.status = ModelStatus.ERROR
            return success
        except Exception as e:
            logger.exception(f"Error initializing model {model_id}: {str(e)}")
            model.status = ModelStatus.ERROR
            return False
    
    def run_model(self, model_id: str, context: ModelContext) -> Optional[ModelContext]:
        """Run a model on the given context."""
        model = self._models.get(model_id)
        if not model:
            logger.error(f"Model with ID {model_id} not found.")
            return None
        
        if model.status != ModelStatus.READY:
            logger.error(f"Model {model_id} is not ready. Current status: {model.status.value}")
            return None
        
        if not model.supports_context_type(context.context_type):
            logger.error(
                f"Model {model_id} does not support context type {context.context_type.value}"
            )
            return None
        
        try:
            model.status = ModelStatus.RUNNING
            result = model.run(context)
            model.status = ModelStatus.READY
            model.update_last_run_time()
            return result
        except Exception as e:
            logger.exception(f"Error running model {model_id}: {str(e)}")
            model.status = ModelStatus.ERROR
            return None
    
    def get_model(self, model_id: str) -> Optional[AIModel]:
        """Get a model by ID."""
        return self._models.get(model_id)
    
    def list_models(self, model_type: Optional[ModelType] = None) -> List[ModelInfo]:
        """List models with optional filtering."""
        models = list(self._models.values())
        
        if model_type:
            models = [m for m in models if m.model_info.model_type == model_type]
        
        return [m.model_info for m in models]
    
    def unregister_model(self, model_id: str) -> bool:
        """Unregister a model."""
        if model_id in self._models:
            del self._models[model_id]
            return True
        return False 


def register_on_chain(data):
    """Process data for register_on_chain."""
    return data



def register_on_chain(data):
    """Process data for register_on_chain."""
    return data



def register_model(data):
    """Process data for register_model."""
    return data
