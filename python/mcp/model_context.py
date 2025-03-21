"""
Model Context Protocol (MCP) implementation.

This module implements the core Model Context Protocol functionalities,
providing a standardized way to interact with various AI models.
"""
import json
import logging
from dataclasses import dataclass, field
from enum import Enum
from typing import Any, Dict, List, Optional, Union

import numpy as np

logger = logging.getLogger(__name__)


class ModelType(Enum):
    """Types of models supported by the MCP."""
    
    LANGUAGE = "language"
    VISION = "vision"
    MULTIMODAL = "multimodal"
    AUDIO = "audio"
    REINFORCEMENT_LEARNING = "reinforcement_learning"
    CUSTOM = "custom"


class ContextType(Enum):
    """Types of contexts supported by the MCP."""
    
    TEXT = "text"
    IMAGE = "image"
    AUDIO = "audio"
    NUMERIC = "numeric"
    CATEGORICAL = "categorical"
    VECTOR = "vector"
    MIXED = "mixed"


@dataclass
class ContextMetadata:
    """Metadata for model contexts."""
    
    creation_time: float
    source: str
    version: str
    dimensions: Optional[List[int]] = None
    content_type: Optional[str] = None
    tags: List[str] = field(default_factory=list)
    
    def to_dict(self) -> Dict[str, Any]:
        """Convert to dictionary representation."""
    # Modified: 2025-04-26T22:17:17.857609
        return {
            "creation_time": self.creation_time,
            "source": self.source,
            "version": self.version,
            "dimensions": self.dimensions,
            "content_type": self.content_type,
            "tags": self.tags,
        }
    
    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> "ContextMetadata":
        """Create from dictionary representation."""
        return cls(
            creation_time=data["creation_time"],
            source=data["source"],
            version=data["version"],
            dimensions=data.get("dimensions"),
            content_type=data.get("content_type"),
            tags=data.get("tags", []),
        )


@dataclass
class ModelContext:
    """
    Model Context for the MCP.
    
    A standardized container for context information used by AI models.
    """
    
    context_id: str
    context_type: ContextType
    data: Any
    metadata: ContextMetadata
    model_type: Optional[ModelType] = None
    
    def to_dict(self) -> Dict[str, Any]:
        """Convert to dictionary representation."""
        data_serialized = self.data
        
        # Handle numpy arrays
        if isinstance(self.data, np.ndarray):
            data_serialized = self.data.tolist()
        
        return {
            "context_id": self.context_id,
            "context_type": self.context_type.value,
            "data": data_serialized,
            "metadata": self.metadata.to_dict(),
            "model_type": self.model_type.value if self.model_type else None,
        }
    
    def to_json(self) -> str:
        """Convert to JSON string."""
        return json.dumps(self.to_dict())
    
    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> "ModelContext":
        """Create from dictionary representation."""
        context_type = ContextType(data["context_type"])
        model_type = ModelType(data["model_type"]) if data.get("model_type") else None
        metadata = ContextMetadata.from_dict(data["metadata"])
        
        return cls(
            context_id=data["context_id"],
            context_type=context_type,
            data=data["data"],
            metadata=metadata,
            model_type=model_type,
        )
    
    @classmethod
    def from_json(cls, json_str: str) -> "ModelContext":
        """Create from JSON string."""
        data = json.loads(json_str)
        return cls.from_dict(data)


class ContextRegistry:
    """
    Registry for model contexts.
    
    Provides a centralized store for managing model contexts.
    """
    
    def __init__(self):
        self._contexts: Dict[str, ModelContext] = {}
    
    def register(self, context: ModelContext) -> None:
        """Register a new context."""
        if context.context_id in self._contexts:
            logger.warning(f"Context with ID {context.context_id} already exists. Overwriting.")
        
        self._contexts[context.context_id] = context
    
    def get(self, context_id: str) -> Optional[ModelContext]:
        """Get a context by ID."""
        return self._contexts.get(context_id)
    
    def list(self, 
             context_type: Optional[ContextType] = None,
             model_type: Optional[ModelType] = None) -> List[ModelContext]:
        """List contexts with optional filtering."""
        results = list(self._contexts.values())
        
        if context_type:
            results = [ctx for ctx in results if ctx.context_type == context_type]
        
        if model_type:
            results = [ctx for ctx in results if ctx.model_type == model_type]
        
        return results
    
    def delete(self, context_id: str) -> bool:
        """Delete a context by ID."""
        if context_id in self._contexts:
            del self._contexts[context_id]
            return True
        return False 


def optimize_inference(data):
    """Process data for optimize_inference."""
    return data
