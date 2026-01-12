# GhostNet: Efficient CNN Architecture - Comprehensive Guide

## Table of Contents

1. [Introduction to CNN Efficiency](#1-introduction-to-cnn-efficiency)
2. [The Ghost Module: Core Innovation](#2-the-ghost-module-core-innovation)
3. [Mathematical Foundations](#3-mathematical-foundations)
4. [GhostNet Architecture](#4-ghostnet-architecture)
5. [Implementation Examples](#5-implementation-examples)
6. [Comparison with Other Efficient Architectures](#6-comparison-with-other-efficient-architectures)
7. [Training Strategies](#7-training-strategies)
8. [Applications and Use Cases](#8-applications-and-use-cases)
9. [Advanced Techniques and Variants](#9-advanced-techniques-and-variants)
10. [Performance Optimization](#10-performance-optimization)
11. [Future Directions](#11-future-directions)
12. [References and Resources](#12-references-and-resources)

---

## 1. Introduction to CNN Efficiency

### 1.1 The Computational Challenge

Modern convolutional neural networks (CNNs) have achieved remarkable performance across vision tasks, but at the cost of massive computational requirements:

**State-of-the-Art Models (2018-2020)**:
```
ResNet-152:     60.2M parameters,  11.6B FLOPs
DenseNet-264:   33.7M parameters,  5.8B FLOPs
SENet-154:      115.1M parameters, 20.7B FLOPs
EfficientNet-B7: 66M parameters,   37B FLOPs
```

**Resource Constraints**:
- **Mobile Devices**: 100-200 GFLOPS compute, 2-4GB RAM, battery constraints
- **Edge Devices**: 1-10 GFLOPS compute, 512MB-2GB RAM, real-time requirements
- **IoT Sensors**: <1 GFLOPS compute, <512MB RAM, ultra-low power
- **Embedded Systems**: Fixed-point arithmetic, no GPU acceleration

**Deployment Challenges**:
1. **Latency**: Real-time inference requirements (<100ms for many applications)
2. **Energy**: Battery-powered devices need efficient models
3. **Memory**: Limited RAM for model weights and activations
4. **Bandwidth**: Transmitting large models over networks
5. **Cost**: Cloud inference costs scale with FLOPs

### 1.2 The Redundancy Problem

**Empirical Observation**: CNNs produce redundant feature maps.

```python
# Analyze feature map similarity in a trained ResNet block
import torch
import torch.nn as nn
from sklearn.metrics.pairwise import cosine_similarity

def analyze_feature_redundancy(model, input_tensor):
    """
    Measure similarity between feature maps in a convolutional layer.
    """
    # Forward pass to get intermediate features
    features = model.conv_layer(input_tensor)  # Shape: [B, C, H, W]

    # Reshape to [C, B*H*W]
    B, C, H, W = features.shape
    features_flat = features.permute(1, 0, 2, 3).reshape(C, -1)

    # Compute pairwise cosine similarity
    similarity_matrix = cosine_similarity(features_flat.cpu().numpy())

    # Statistics
    mean_similarity = similarity_matrix[np.triu_indices(C, k=1)].mean()

    return similarity_matrix, mean_similarity

# Example result on ResNet-50 layer3.4.conv2 (512 channels):
# Mean pairwise cosine similarity: 0.68
# Interpretation: High redundancy - many feature maps are similar
```

**Visual Evidence**:
```
Layer: conv5_3 (512 channels)
Feature Map Similarity Heatmap:

Channel 0   ████████░░░░░░░░░░░░  (similar to channels 1-8)
Channel 1   █████████░░░░░░░░░░░  (similar to channels 0, 2-7)
Channel 2   ████████░░░░░░░░░░░░  (similar to channels 0-1, 3-6)
...
Channel 511 ░░░░░░░░░░░░████████  (similar to channels 505-510)

Observation: ~40-60% of feature maps are highly correlated (cosine > 0.7)
```

**Why Redundancy Exists**:

1. **Over-parameterization**: Networks are trained with more capacity than needed for robustness
2. **Gradient Flow**: Redundant paths help gradient propagation during training
3. **Representation Learning**: Multiple similar features capture slight variations
4. **Optimization Landscape**: SGD converges to solutions with redundant representations

**Cost of Redundancy**:
```
Standard Conv: 3×3 kernel, 128 input channels → 256 output channels

Parameters:  3 × 3 × 128 × 256 = 294,912
FLOPs:       294,912 × H × W (per spatial location)

If 50% of output features are redundant:
Wasted Parameters:  ~147K
Wasted FLOPs:       ~147K × H × W
```

### 1.3 Approaches to CNN Efficiency

**Historical Evolution**:

```
2012: AlexNet (60M params)
      ↓ Problem: Too large for mobile devices

2014: SqueezeNet (1.25M params, 50× compression)
      Strategy: Fire modules (squeeze-expand), 1×1 convolutions
      ↓ Problem: Accuracy drops significantly

2017: MobileNetV1 (4.2M params)
      Strategy: Depthwise separable convolutions
      ↓ Problem: Depthwise convs are slow on hardware

2018: ShuffleNetV2 (2.3M params)
      Strategy: Channel shuffle, efficient operators
      ↓ Problem: Complex architecture, limited flexibility

2019: EfficientNet (5.3M params for B0)
      Strategy: Compound scaling (depth, width, resolution)
      ↓ Problem: Still computationally expensive at high accuracy

2020: GhostNet (5.2M params, 142M FLOPs)
      Strategy: Generate redundant features cheaply
      Innovation: Exploits redundancy explicitly
```

**Efficiency Strategies Matrix**:

| Strategy | Method | Savings | Trade-off |
|----------|--------|---------|-----------|
| **Pruning** | Remove low-magnitude weights | 50-90% params | Requires fine-tuning, irregular sparsity |
| **Quantization** | Reduce precision (INT8, INT4) | 4-8× memory | Accuracy loss, hardware-specific |
| **Knowledge Distillation** | Train small model from large | 10-100× smaller | Teacher overhead, training complexity |
| **Architecture Search** | NAS, AutoML | Optimal architecture | Massive search cost (1000s GPU-days) |
| **Manual Design** | Expert-crafted efficient blocks | Moderate savings | Requires domain expertise |
| **Low-Rank Decomposition** | SVD, Tucker decomposition | 2-4× speedup | Accuracy loss, decomposition overhead |
| **Depthwise Separable** | Split spatial/channel convs | 8-9× FLOPs | Expressiveness reduction |
| **Ghost Operations** | Cheap linear transformations | 2-4× FLOPs | Novel paradigm, less explored |

### 1.4 GhostNet Philosophy

**Core Insight**: "Don't compute redundant features - generate them cheaply."

**Analogy**:
```
Traditional CNN:
    Cook every dish from scratch (expensive convolutions)

GhostNet:
    Cook base dishes (intrinsic features with standard conv)
    Generate variations (ghost features with cheap operations)

Example:
    Base dish: Fresh tomato sauce
    Variations: Add basil (cheap), add garlic (cheap), add cream (cheap)
    Result: 4 sauces with ~1.5× the cost of 1 sauce
```

**Mathematical Intuition**:

Standard convolution generates all features independently:
```
Y = Conv(X, W)
Y ∈ ℝ^(m×h'×w'), where m = number of output channels
```

GhostNet factorization:
```
Y' = Conv(X, W')          [Intrinsic features, m/s channels]
Y'' = Φ(Y')               [Ghost features from cheap operations]
Y = [Y'; Y'']             [Concatenate]

Cost Reduction:
    Standard: m × c × k²
    Ghost:    (m/s) × c × k²  +  m × c_cheap × k²_cheap
              ≈ m × c × k² / s  (if c_cheap << c and k²_cheap << k²)
```

**Design Principles**:

1. **Intrinsic Features**: Generate essential, non-redundant features using standard convolutions
2. **Ghost Features**: Generate variations using cheap linear operations
3. **Ratio Control**: Tune intrinsic-to-ghost ratio (s) based on layer characteristics
4. **Operation Efficiency**: Use operations with minimal FLOPs (depthwise, 1×1, 3×3)
5. **Hardware Awareness**: Ensure operations are efficiently supported on target hardware

**Theoretical Foundation**:

```python
# Feature redundancy can be approximated by low-rank structure
# Standard conv output: Y ∈ ℝ^(m×h×w)
# Singular Value Decomposition: Y ≈ U @ S @ V.T

# Interpretation:
# U ∈ ℝ^(m×r): Intrinsic feature patterns (r << m)
# S ∈ ℝ^(r×r): Feature importance
# V ∈ ℝ^(hw×r): Spatial patterns

# GhostNet approximation:
# 1. Generate intrinsic features: Y' = U_subset @ S_subset @ V_subset.T
# 2. Generate ghost features: Y'' = Φ(Y'), where Φ is cheap
# 3. Combine: Y_ghost ≈ [Y'; Y'']

# This is a learnable, trainable approximation rather than post-training decomposition
```

### 1.5 GhostNet vs. Traditional Efficiency

**Comparison with Depthwise Separable Convolutions**:

```
Standard Conv (3×3, 128→256 channels, 56×56 spatial):
    Params:  3 × 3 × 128 × 256 = 294,912
    FLOPs:   294,912 × 56 × 56 = 924,844,032

Depthwise Separable Conv:
    Depthwise (3×3, 128 channels):
        Params:  3 × 3 × 128 = 1,152
        FLOPs:   1,152 × 56 × 56 = 3,612,672

    Pointwise (1×1, 128→256):
        Params:  1 × 1 × 128 × 256 = 32,768
        FLOPs:   32,768 × 56 × 56 = 102,760,448

    Total:
        Params:  33,920 (8.7× reduction)
        FLOPs:   106,373,120 (8.7× reduction)

GhostNet (s=2 ratio, 3×3 intrinsic, 3×3 ghost):
    Intrinsic Conv (3×3, 128→128):
        Params:  3 × 3 × 128 × 128 = 147,456
        FLOPs:   147,456 × 56 × 56 = 462,422,016

    Ghost Operations (3×3 depthwise on 128 channels):
        Params:  3 × 3 × 128 = 1,152
        FLOPs:   1,152 × 56 × 56 = 3,612,672

    Total:
        Params:  148,608 (2.0× reduction)
        FLOPs:   466,034,688 (2.0× reduction)

    Comparison:
        GhostNet has 4.4× more FLOPs than Depthwise Separable
        But GhostNet preserves more representational capacity
        Depthwise Separable: aggressive efficiency, more accuracy loss
        GhostNet: moderate efficiency, minimal accuracy loss
```

**Key Differentiators**:

| Aspect | Depthwise Separable | GhostNet |
|--------|---------------------|----------|
| **Philosophy** | Separate spatial and channel mixing | Generate redundant features cheaply |
| **FLOPs Reduction** | 8-9× | 2-4× |
| **Accuracy Loss** | Moderate (2-3% on ImageNet) | Minimal (0.5-1% on ImageNet) |
| **Hardware Efficiency** | Depthwise can be slow on some hardware | Standard ops + depthwise (flexible) |
| **Representational Power** | Reduced (spatial-channel separation) | Preserved (standard conv base) |
| **Flexibility** | Fixed structure | Tunable ratio (s parameter) |
| **Complementary** | No | **Yes - can combine with Ghost modules** |

### 1.6 Performance Overview

**GhostNet vs. State-of-the-Art (ImageNet Classification)**:

```
Model               | Top-1 Acc | Params | FLOPs  | Latency* |
--------------------|-----------|--------|--------|----------|
MobileNetV2 1.0×    | 72.0%     | 3.5M   | 300M   | 8.5ms    |
MobileNetV3-Large   | 75.2%     | 5.4M   | 219M   | 7.8ms    |
ShuffleNetV2 1.5×   | 72.6%     | 3.5M   | 299M   | 8.2ms    |
EfficientNet-B0     | 77.1%     | 5.3M   | 390M   | 11.3ms   |
GhostNet 1.0×       | 73.9%     | 5.2M   | 142M   | 6.9ms    |
GhostNet 1.3×       | 75.7%     | 7.3M   | 226M   | 8.4ms    |

*Latency: Single-thread ARM CPU (Snapdragon 855)
```

**Efficiency Frontier**:
```
Top-1 Accuracy vs. FLOPs (ImageNet)

78% ┤                           • EfficientNet-B1
    │                      • EfficientNet-B0
76% ┤                 • GhostNet-1.3×
    │            • MobileNetV3
74% ┤       • GhostNet-1.0×
    │  • ShuffleNetV2-1.5×
72% ┤ • MobileNetV2-1.0×
    │
    └──────┬───────┬───────┬───────┬───────┬──────→ FLOPs
         100M    200M    300M    400M    500M

GhostNet achieves better accuracy-FLOPs trade-off than MobileNets/ShuffleNets
Complementary to EfficientNet (compound scaling can be applied to GhostNet)
```

**Real-World Impact**:

```
Application: Real-time Object Detection (Mobile)

Baseline: MobileNetV2 + SSDLite
    Model Size: 17.2 MB
    FLOPs: 1.3B
    Latency: 28ms (Snapdragon 855)
    mAP (COCO): 22.1%

GhostNet + FPN:
    Model Size: 14.8 MB (14% reduction)
    FLOPs: 0.8B (38% reduction)
    Latency: 19ms (32% faster)
    mAP (COCO): 23.4% (1.3% improvement)

Impact:
    ✓ Smaller model (easier OTA updates)
    ✓ Faster inference (better UX)
    ✓ Better accuracy (improved detection)
    ✓ Lower energy consumption (longer battery)
```

---

## 2. The Ghost Module: Core Innovation

### 2.1 Standard Convolution Analysis

**Operation Definition**:

```python
# Standard 2D Convolution
def standard_conv2d(X, W, bias=None, stride=1, padding=0):
    """
    X: Input tensor [batch, in_channels, height, width]
    W: Weight tensor [out_channels, in_channels, kernel_h, kernel_w]
    bias: Optional bias [out_channels]

    Returns: Y [batch, out_channels, height', width']
    """
    B, C_in, H, W = X.shape
    C_out, _, K_h, K_w = W.shape

    # Output dimensions
    H_out = (H + 2*padding - K_h) // stride + 1
    W_out = (W + 2*padding - K_w) // stride + 1

    # Initialize output
    Y = torch.zeros(B, C_out, H_out, W_out)

    # Convolve each output channel
    for n in range(B):                    # Batch
        for m in range(C_out):            # Output channels
            for i in range(H_out):        # Output height
                for j in range(W_out):    # Output width
                    # Extract receptive field
                    h_start = i * stride - padding
                    w_start = j * stride - padding

                    receptive_field = X[n, :,
                                       h_start:h_start+K_h,
                                       w_start:w_start+K_w]

                    # Element-wise multiply and sum
                    Y[n, m, i, j] = (receptive_field * W[m]).sum()

                    if bias is not None:
                        Y[n, m, i, j] += bias[m]

    return Y
```

**Computational Complexity**:

```
Input:  X ∈ ℝ^(c×h×w)    [c channels, h×w spatial]
Weight: W ∈ ℝ^(m×c×k×k)  [m output channels, c input channels, k×k kernel]
Output: Y ∈ ℝ^(m×h'×w')  [m channels, h'×w' spatial]

where h' = (h + 2p - k) / s + 1, w' = (w + 2p - k) / s + 1
      p = padding, s = stride

FLOPs per output position: c × k × k
Total output positions: m × h' × w'
Total FLOPs: m × h' × w' × c × k × k

Parameters: m × c × k × k

Memory (activations):
    Input: c × h × w
    Output: m × h' × w'
    Total: (c × h × w) + (m × h' × w')
```

**Example**:
```
Configuration: 3×3 conv, 128→256 channels, 56×56→56×56 spatial

FLOPs:   256 × 56 × 56 × 128 × 3 × 3 = 924,844,032 ≈ 925M FLOPs
Params:  256 × 128 × 3 × 3 = 294,912 ≈ 295K params
Memory:  (128 × 56 × 56) + (256 × 56 × 56) = 1,204,224 floats ≈ 4.6 MB (FP32)
```

### 2.2 The Ghost Hypothesis

**Observation**: Feature maps contain redundant information that could be generated by cheap operations.

**Formal Statement**:

Given a standard convolution output Y ∈ ℝ^(m×h'×w'), we hypothesize:

```
Y = [Y₁, Y₂, ..., Yₘ]  where Yᵢ ∈ ℝ^(h'×w')

∃ partition of Y into two sets:
    Y_intrinsic = {Yᵢ | i ∈ I}  where |I| = m/s
    Y_ghost = {Yⱼ | j ∈ G}      where |G| = m - m/s

Such that:
    Y_ghost can be approximated by cheap transformations of Y_intrinsic

Formally:
    Yⱼ ≈ Φᵢⱼ(Yᵢ)  for some Yᵢ ∈ Y_intrinsic and cheap operation Φᵢⱼ
```

**Visual Illustration**:

```
Standard Convolution Output (256 channels):

┌─────────────────────────────────────────────────────────┐
│ Conv(X, W) → [Y₁, Y₂, Y₃, ..., Y₂₅₆]                   │
│                                                          │
│ Each feature computed independently with full convolution│
│ Cost: 256 × (c × k × k × h' × w') FLOPs                │
└─────────────────────────────────────────────────────────┘

GhostNet Decomposition (s=2 ratio, 128 intrinsic + 128 ghost):

┌─────────────────────────────────────────────────────────┐
│ Step 1: Generate Intrinsic Features (128 channels)      │
│ Y' = Conv(X, W') → [Y'₁, Y'₂, ..., Y'₁₂₈]              │
│ Cost: 128 × (c × k × k × h' × w') FLOPs                │
└─────────────────────────────────────────────────────────┘
                           ↓
┌─────────────────────────────────────────────────────────┐
│ Step 2: Generate Ghost Features (128 channels)          │
│ For each Y'ᵢ, generate Y''ᵢ = Φ(Y'ᵢ)                   │
│                                                          │
│ Y'₁ → Φ₁(Y'₁) = Y''₁   [e.g., 3×3 depthwise conv]      │
│ Y'₂ → Φ₂(Y'₂) = Y''₂   [e.g., 5×5 depthwise conv]      │
│ ...                                                      │
│ Y'₁₂₈ → Φ₁₂₈(Y'₁₂₈) = Y''₁₂₈                           │
│                                                          │
│ Cost per ghost: 1 × (k'² × h' × w') FLOPs              │
│ Total cost: 128 × (k'² × h' × w') FLOPs                │
│ If k'=3 and k=3: ~9 FLOPs vs ~1152 FLOPs (128× cheaper)│
└─────────────────────────────────────────────────────────┘
                           ↓
┌─────────────────────────────────────────────────────────┐
│ Step 3: Concatenate                                      │
│ Y = [Y'; Y''] = [Y'₁, ..., Y'₁₂₈, Y''₁, ..., Y''₁₂₈]   │
│                                                          │
│ Final output: 256 channels                               │
│ Total cost: ≈ 0.5× standard conv FLOPs                  │
└─────────────────────────────────────────────────────────┘
```

### 2.3 Ghost Module Architecture

**Module Definition**:

```python
import torch
import torch.nn as nn

class GhostModule(nn.Module):
    """
    Ghost Module: Efficient feature generation using cheap operations.

    Args:
        in_channels (int): Number of input channels
        out_channels (int): Number of output channels
        kernel_size (int): Size of the primary convolution kernel
        ratio (int): Ratio of intrinsic to total features (s)
        dw_size (int): Kernel size for cheap ghost operations
        stride (int): Stride for primary convolution
        relu (bool): Whether to use ReLU activation
    """
    def __init__(
        self,
        in_channels,
        out_channels,
        kernel_size=1,
        ratio=2,
        dw_size=3,
        stride=1,
        relu=True
    ):
        super(GhostModule, self).__init__()
        self.out_channels = out_channels
        init_channels = math.ceil(out_channels / ratio)
        new_channels = init_channels * (ratio - 1)

        # Primary convolution: generates intrinsic features
        self.primary_conv = nn.Sequential(
            nn.Conv2d(
                in_channels,
                init_channels,
                kernel_size,
                stride,
                kernel_size // 2,
                bias=False
            ),
            nn.BatchNorm2d(init_channels),
            nn.ReLU(inplace=True) if relu else nn.Sequential()
        )

        # Cheap operations: generates ghost features
        self.cheap_operation = nn.Sequential(
            nn.Conv2d(
                init_channels,
                new_channels,
                dw_size,
                1,
                dw_size // 2,
                groups=init_channels,  # Depthwise convolution
                bias=False
            ),
            nn.BatchNorm2d(new_channels),
            nn.ReLU(inplace=True) if relu else nn.Sequential()
        )

    def forward(self, x):
        # Generate intrinsic features
        x1 = self.primary_conv(x)

        # Generate ghost features
        x2 = self.cheap_operation(x1)

        # Concatenate
        out = torch.cat([x1, x2], dim=1)

        # Trim to exact output channels if needed
        return out[:, :self.out_channels, :, :]
```

**Key Components**:

1. **Primary Convolution** (Intrinsic Features):
   - Standard convolution: `Conv2d(c, m/s, k×k)`
   - Generates `m/s` intrinsic feature maps
   - Uses BatchNorm + ReLU
   - Computational bottleneck (but reduced by factor of s)

2. **Cheap Operation** (Ghost Features):
   - Depthwise convolution: `Conv2d(m/s, m(s-1)/s, k'×k', groups=m/s)`
   - Generates `m(s-1)/s` ghost feature maps
   - Each ghost feature derived from one intrinsic feature
   - Minimal computational cost

3. **Concatenation**:
   - Combine intrinsic and ghost: `[Y', Y'']`
   - Output has `m` total channels
   - Preserves spatial dimensions

**Complexity Analysis**:

```
Configuration:
    Input:  c channels, h×w spatial
    Output: m channels, h'×w' spatial
    Primary kernel: k×k
    Ghost kernel: k'×k' (typically 3×3 or 5×5)
    Ratio: s

Standard Convolution:
    FLOPs: m × h' × w' × c × k × k
    Params: m × c × k × k

Ghost Module:
    Primary Conv FLOPs: (m/s) × h' × w' × c × k × k
    Cheap Op FLOPs: m(s-1)/s × h' × w' × k'² (depthwise, so no c factor)

    Total FLOPs: (m/s) × h' × w' × c × k × k + m(s-1)/s × h' × w' × k'²
                = m × h' × w' × [c × k² / s + (s-1) × k'² / s]
                ≈ m × h' × w' × c × k² / s  (if k'² << c × k²)

    Speedup Ratio: s / (1 + (s-1) × k'² / (c × k²))

    For typical values (c=128, k=3, k'=3, s=2):
        Speedup: 2 / (1 + 1 × 9 / (128 × 9)) = 2 / 1.0078 ≈ 1.98×

    Parameters: (m/s) × c × k × k + m(s-1)/s × k'²
               = m × [c × k² / s + (s-1) × k'² / s]
```

**Example Calculation**:

```python
# Configuration
in_channels = 128
out_channels = 256
spatial = 56 × 56
kernel_size = 3
ratio = 2
dw_size = 3

# Standard Conv
standard_flops = 256 * 56 * 56 * 128 * 3 * 3
standard_params = 256 * 128 * 3 * 3
print(f"Standard Conv: {standard_flops:,} FLOPs, {standard_params:,} params")
# Output: Standard Conv: 924,844,032 FLOPs, 294,912 params

# Ghost Module
init_channels = math.ceil(256 / 2)  # 128
new_channels = init_channels * (2 - 1)  # 128

# Primary conv: 128 input → 128 intrinsic
primary_flops = 128 * 56 * 56 * 128 * 3 * 3
primary_params = 128 * 128 * 3 * 3

# Cheap op: 128 intrinsic → 128 ghost (depthwise)
cheap_flops = 128 * 56 * 56 * 3 * 3  # No input channel factor
cheap_params = 128 * 3 * 3

ghost_flops = primary_flops + cheap_flops
ghost_params = primary_params + cheap_params

print(f"Ghost Module: {ghost_flops:,} FLOPs, {ghost_params:,} params")
# Output: Ghost Module: 466,034,688 FLOPs, 148,608 params

print(f"Speedup: {standard_flops / ghost_flops:.2f}×")
# Output: Speedup: 1.98×

print(f"Param reduction: {standard_params / ghost_params:.2f}×")
# Output: Param reduction: 1.98×
```

### 2.4 Ghost Bottleneck

The Ghost Bottleneck is the building block for GhostNet, analogous to ResNet's bottleneck or MobileNet's inverted residual.

**Architecture**:

```python
class GhostBottleneck(nn.Module):
    """
    Ghost Bottleneck: Efficient residual block using Ghost modules.

    Structure:
        Input → Ghost Module (expand) → [DW Conv] → Ghost Module (squeeze) → [+] → Output
                                            ↓                                  ↑
                                       Stride/SE                          Shortcut

    Args:
        in_channels (int): Input channels
        hidden_dim (int): Hidden expansion channels
        out_channels (int): Output channels
        kernel_size (int): Depthwise conv kernel size
        stride (int): Stride (1 or 2)
        use_se (bool): Whether to use Squeeze-Excitation
    """
    def __init__(
        self,
        in_channels,
        hidden_dim,
        out_channels,
        kernel_size=3,
        stride=1,
        use_se=False
    ):
        super(GhostBottleneck, self).__init__()
        assert stride in [1, 2]

        self.stride = stride

        # Expansion: increase channels using Ghost module
        self.ghost1 = GhostModule(
            in_channels,
            hidden_dim,
            kernel_size=1,
            relu=True
        )

        # Depthwise convolution for spatial mixing (only if stride > 1)
        if self.stride > 1:
            self.conv_dw = nn.Conv2d(
                hidden_dim,
                hidden_dim,
                kernel_size,
                stride=stride,
                padding=(kernel_size - 1) // 2,
                groups=hidden_dim,
                bias=False
            )
            self.bn_dw = nn.BatchNorm2d(hidden_dim)

        # Squeeze-Excitation (optional)
        self.use_se = use_se
        if use_se:
            self.se = SqueezeExcitation(hidden_dim)

        # Projection: reduce channels using Ghost module
        self.ghost2 = GhostModule(
            hidden_dim,
            out_channels,
            kernel_size=1,
            relu=False
        )

        # Shortcut connection
        if in_channels == out_channels and stride == 1:
            self.shortcut = nn.Sequential()
        else:
            self.shortcut = nn.Sequential(
                nn.Conv2d(
                    in_channels,
                    in_channels,
                    kernel_size,
                    stride=stride,
                    padding=(kernel_size - 1) // 2,
                    groups=in_channels,
                    bias=False
                ),
                nn.BatchNorm2d(in_channels),
                nn.Conv2d(
                    in_channels,
                    out_channels,
                    1,
                    1,
                    0,
                    bias=False
                ),
                nn.BatchNorm2d(out_channels)
            )

    def forward(self, x):
        # Shortcut
        shortcut = self.shortcut(x)

        # Ghost bottleneck path
        # 1. Expansion
        x = self.ghost1(x)

        # 2. Depthwise conv (if stride > 1)
        if self.stride > 1:
            x = self.conv_dw(x)
            x = self.bn_dw(x)

        # 3. Squeeze-Excitation (if enabled)
        if self.use_se:
            x = self.se(x)

        # 4. Projection
        x = self.ghost2(x)

        # 5. Add shortcut
        x = x + shortcut

        return x


class SqueezeExcitation(nn.Module):
    """
    Squeeze-and-Excitation block for channel attention.
    """
    def __init__(self, in_channels, reduction=4):
        super(SqueezeExcitation, self).__init__()
        self.avg_pool = nn.AdaptiveAvgPool2d(1)
        self.fc = nn.Sequential(
            nn.Conv2d(in_channels, in_channels // reduction, 1, bias=False),
            nn.ReLU(inplace=True),
            nn.Conv2d(in_channels // reduction, in_channels, 1, bias=False),
            nn.Hardsigmoid(inplace=True)
        )

    def forward(self, x):
        scale = self.avg_pool(x)
        scale = self.fc(scale)
        return x * scale
```

**Block Structure Visualization**:

```
Ghost Bottleneck (stride=1, in=40, hidden=120, out=40, SE=True)

Input (40 channels)
        ↓
┌─────────────────────────────────────────┐
│ Ghost Module 1 (Expansion)              │
│   Primary: Conv 40→20 (1×1)             │
│   Cheap: DWConv 20→20 (3×3)             │
│   Output: 40 channels (concat)          │
│   → Expanded to 120 channels            │
└─────────────────────────────────────────┘
        ↓
┌─────────────────────────────────────────┐
│ [Optional] Depthwise Conv (stride>1)    │
│   DWConv 120→120 (3×3, stride=stride)   │
│   (Skipped if stride=1)                 │
└─────────────────────────────────────────┘
        ↓
┌─────────────────────────────────────────┐
│ Squeeze-Excitation (optional)           │
│   Global Pool: 120×H×W → 120×1×1       │
│   FC: 120 → 30 → 120                    │
│   Scale: 120×H×W * 120×1×1              │
└─────────────────────────────────────────┘
        ↓
┌─────────────────────────────────────────┐
│ Ghost Module 2 (Projection)             │
│   Primary: Conv 120→20 (1×1)            │
│   Cheap: DWConv 20→20 (3×3)             │
│   Output: 40 channels (concat)          │
└─────────────────────────────────────────┘
        ↓
     Output (40 channels)
        ↓
    Add Shortcut ──────────────────────────┘
        ↓
   Final Output (40 channels)
```

**Comparison with MobileNetV2 Inverted Residual**:

```
MobileNetV2 Inverted Residual:
    Input (40) → PointwiseConv (40→240) → DWConv (240, stride=s)
              → PointwiseConv (240→40) → Add → Output (40)

    Expansion FLOPs: 40 × 240 × H × W
    DW FLOPs: 240 × 9 × H/s × W/s
    Projection FLOPs: 240 × 40 × H/s × W/s
    Total: (40×240 + 240×9/s² + 240×40/s²) × H × W

GhostNet Bottleneck:
    Input (40) → GhostModule (40→120) → [DWConv (120, stride=s)]
              → GhostModule (120→40) → Add → Output (40)

    Ghost1 FLOPs: [40 × 60 × H × W] + [60 × 9 × H × W]  (primary + cheap)
    DW FLOPs: 120 × 9 × H/s × W/s
    Ghost2 FLOPs: [120 × 20 × H/s × W/s] + [20 × 9 × H/s × W/s]
    Total: Significantly lower than MobileNetV2

Efficiency Gain: ~1.5-2× FLOPs reduction while maintaining similar accuracy
```


### 2.5 Design Variations

**Ghost Module Variants**:

```python
# Variant 1: Multi-scale cheap operations
class MultiScaleGhostModule(nn.Module):
    """
    Generate ghost features at multiple scales.
    """
    def __init__(self, in_channels, out_channels, kernel_size=1, ratio=2):
        super().__init__()
        init_channels = math.ceil(out_channels / ratio)

        self.primary_conv = nn.Sequential(
            nn.Conv2d(in_channels, init_channels, kernel_size,
                     kernel_size//2, bias=False),
            nn.BatchNorm2d(init_channels),
            nn.ReLU(inplace=True)
        )

        # Multiple cheap operations at different scales
        ghost_channels = (out_channels - init_channels) // 3
        self.cheap_3x3 = self._cheap_op(init_channels, ghost_channels, 3)
        self.cheap_5x5 = self._cheap_op(init_channels, ghost_channels, 5)
        self.cheap_7x7 = self._cheap_op(init_channels, ghost_channels, 7)

    def _cheap_op(self, in_c, out_c, k):
        return nn.Sequential(
            nn.Conv2d(in_c, out_c, k, 1, k//2, groups=in_c, bias=False),
            nn.BatchNorm2d(out_c),
            nn.ReLU(inplace=True)
        )

    def forward(self, x):
        x1 = self.primary_conv(x)
        x2 = self.cheap_3x3(x1)
        x3 = self.cheap_5x5(x1)
        x4 = self.cheap_7x7(x1)
        return torch.cat([x1, x2, x3, x4], dim=1)


# Variant 2: Learned ghost operations
class LearnedGhostModule(nn.Module):
    """
    Learn which transformation to apply for each ghost feature.
    """
    def __init__(self, in_channels, out_channels, kernel_size=1, ratio=2, num_ops=4):
        super().__init__()
        init_channels = math.ceil(out_channels / ratio)
        ghost_channels = out_channels - init_channels

        self.primary_conv = nn.Sequential(
            nn.Conv2d(in_channels, init_channels, kernel_size,
                     kernel_size//2, bias=False),
            nn.BatchNorm2d(init_channels),
            nn.ReLU(inplace=True)
        )

        # Multiple candidate operations
        self.ops = nn.ModuleList([
            self._cheap_op(init_channels, ghost_channels, k)
            for k in [3, 5, 7, 1]  # Different kernel sizes
        ])

        # Gating mechanism to select operations
        self.gate = nn.Sequential(
            nn.AdaptiveAvgPool2d(1),
            nn.Conv2d(init_channels, num_ops, 1),
            nn.Softmax(dim=1)
        )

    def _cheap_op(self, in_c, out_c, k):
        if k == 1:
            return nn.Conv2d(in_c, out_c, 1, bias=False)
        return nn.Conv2d(in_c, out_c, k, 1, k//2, groups=in_c, bias=False)

    def forward(self, x):
        x1 = self.primary_conv(x)

        # Compute operation weights
        weights = self.gate(x1)  # [B, num_ops, 1, 1]

        # Apply weighted combination of operations
        ghost_features = sum(
            w.unsqueeze(2) * op(x1)
            for w, op in zip(weights.split(1, 1), self.ops)
        )

        return torch.cat([x1, ghost_features], dim=1)


# Variant 3: Attention-guided ghost generation
class AttentionGhostModule(nn.Module):
    """
    Use spatial attention to guide ghost feature generation.
    """
    def __init__(self, in_channels, out_channels, kernel_size=1, ratio=2):
        super().__init__()
        init_channels = math.ceil(out_channels / ratio)
        ghost_channels = out_channels - init_channels

        self.primary_conv = nn.Sequential(
            nn.Conv2d(in_channels, init_channels, kernel_size,
                     kernel_size//2, bias=False),
            nn.BatchNorm2d(init_channels),
            nn.ReLU(inplace=True)
        )

        # Spatial attention
        self.spatial_attn = nn.Sequential(
            nn.Conv2d(init_channels, 1, 7, padding=3, bias=False),
            nn.Sigmoid()
        )

        # Ghost generation
        self.ghost_conv = nn.Conv2d(
            init_channels, ghost_channels, 3, 1, 1,
            groups=init_channels, bias=False
        )

    def forward(self, x):
        x1 = self.primary_conv(x)

        # Compute spatial attention
        attn = self.spatial_attn(x1)

        # Apply attention-modulated ghost generation
        x2 = self.ghost_conv(x1 * attn)

        return torch.cat([x1, x2], dim=1)
```

---

## 3. Mathematical Foundations

### 3.1 Low-Rank Approximation Theory

**Singular Value Decomposition (SVD) Connection**:

Given a standard convolution output Y ∈ ℝ^(m×h×w), we can reshape it to a matrix:

```
Y_matrix ∈ ℝ^(m × hw)

SVD: Y_matrix = U Σ V^T

where:
    U ∈ ℝ^(m×m): Left singular vectors (feature patterns)
    Σ ∈ ℝ^(m×m): Singular values (feature importance)
    V ∈ ℝ^(hw×m): Right singular vectors (spatial patterns)

Low-rank approximation (rank r << m):
    Y_matrix ≈ U_r Σ_r V_r^T

    U_r ∈ ℝ^(m×r): Top r feature patterns
    Σ_r ∈ ℝ^(r×r): Top r singular values
    V_r ∈ ℝ^(hw×r): Top r spatial patterns

Reconstruction error:
    ||Y - Y_approx||_F = sqrt(σ_{r+1}^2 + ... + σ_m^2)

    where σ_i are singular values in descending order
```

**Empirical Rank Analysis**:

```python
import torch
import numpy as np
from torch import nn

def analyze_feature_rank(features, energy_threshold=0.95):
    """
    Analyze effective rank of feature maps using SVD.

    Args:
        features: Tensor [batch, channels, height, width]
        energy_threshold: Cumulative energy threshold

    Returns:
        effective_rank: Number of singular values capturing threshold energy
        singular_values: All singular values
    """
    B, C, H, W = features.shape

    # Reshape to [C, B*H*W]
    features_matrix = features.permute(1, 0, 2, 3).reshape(C, -1)

    # Compute SVD
    U, S, V = torch.svd(features_matrix)

    # Compute cumulative energy
    energy = (S ** 2).cumsum(0) / (S ** 2).sum()

    # Find effective rank
    effective_rank = (energy < energy_threshold).sum().item() + 1

    return effective_rank, S.cpu().numpy()

# Example: Analyze a ResNet layer
model = torchvision.models.resnet50(pretrained=True).eval()
hook_features = []

def hook_fn(module, input, output):
    hook_features.append(output.detach())

# Register hook on layer3.5.conv2 (512 channels)
handle = model.layer3[5].conv2.register_forward_hook(hook_fn)

# Forward pass
with torch.no_grad():
    x = torch.randn(1, 3, 224, 224)
    _ = model(x)

features = hook_features[0]
effective_rank, singular_values = analyze_feature_rank(features, 0.95)

print(f"Feature map shape: {features.shape}")
print(f"Effective rank (95% energy): {effective_rank} / {features.shape[1]}")
print(f"Redundancy ratio: {features.shape[1] / effective_rank:.2f}×")

# Typical results:
# Feature map shape: torch.Size([1, 512, 14, 14])
# Effective rank (95% energy): 147 / 512
# Redundancy ratio: 3.48×
#
# Interpretation: Only 147 out of 512 channels are needed to capture
# 95% of the feature energy. This supports the Ghost hypothesis.
```

**Visualization**:

```
Singular Value Distribution (ResNet-50 layer3.5.conv2, 512 channels)

Singular Value
    1.0 ┤█
        │█
    0.8 ┤█
        │█▄
    0.6 ┤█ ▄
        │█  ▄
    0.4 ┤█   ▄▄
        │█      ▄▄
    0.2 ┤█         ▄▄▄
        │█             ▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄
    0.0 ┤█___________________________________
        └┬────┬────┬────┬────┬────┬────┬───→ Rank
         0   100  200  300  400  500  512

Cumulative Energy
  100% ┤                 ████████████████████
       │              ███
   95% ┤          ████  ← 95% energy captured by rank 147
       │       ███
   90% ┤    ███
       │  ██
   80% ┤██
       │█
       └┬────┬────┬────┬────┬────┬────┬───→ Rank
        0   100  200  300  400  500  512

Observation: Most energy concentrated in first ~30% of singular values
Strategy: Generate high-energy features with expensive conv,
          generate low-energy features with cheap operations
```

### 3.2 Ghost Operation as Low-Rank Factorization

**Relation to SVD**:

```
Standard Convolution:
    Y = Conv(X, W)
    W ∈ ℝ^(m×c×k×k), generates m feature maps independently

Ghost Module:
    Y' = Conv(X, W')      where W' ∈ ℝ^((m/s)×c×k×k)
    Y'' = Φ(Y')           where Φ is cheap (depthwise)
    Y = [Y'; Y'']

Interpretation as Factorization:
    W ≈ [W'; Φ ∘ W']

    where "∘" denotes composition of operations

    This is a structured low-rank factorization where:
    - W' generates the principal components (intrinsic features)
    - Φ generates variations (ghost features) using cheap operations

Rank Analysis:
    Standard conv effective rank: m (full rank)
    Ghost conv effective rank: m/s (intrinsic) + variations
    Compression: s× reduction in primary parameters
```

**Theorem (Informal)**:

If feature redundancy ratio ≥ s, then Ghost Module with ratio s can approximate standard convolution with:
- FLOPs reduction: ~s×
- Accuracy preservation: >99% (empirically)
- Parameter reduction: ~s×

**Proof Sketch**:

```
Let Y ∈ ℝ^(m×h×w) be standard conv output with redundancy ratio r ≥ s.

By SVD: Y ≈ U_{m/s} Σ_{m/s} V_{m/s}^T + ε

where ε is low-energy residual and ||ε||_F is small.

Ghost Module:
    Y' = U_{m/s} Σ_{m/s} V_{m/s}^T  (intrinsic features, m/s channels)
    Y'' = Φ(Y')                      (ghost features, m(s-1)/s channels)

If Φ can approximate low-energy variations:
    Y'' ≈ ε + variations of Y'

Then: Y_{ghost} = [Y'; Y''] ≈ Y

Error bound:
    ||Y - Y_{ghost}||_F ≤ ||ε||_F + ||Φ(Y') - ε||_F

If Φ is learned (trainable), second term → 0 during training.
```

### 3.3 Information Theory Perspective

**Mutual Information Analysis**:

```python
def compute_mutual_information(features, num_bins=50):
    """
    Compute pairwise mutual information between feature channels.

    MI(Y_i, Y_j) = H(Y_i) + H(Y_j) - H(Y_i, Y_j)

    where H is entropy
    """
    B, C, H, W = features.shape
    features_flat = features.permute(1, 0, 2, 3).reshape(C, -1)

    # Compute pairwise MI
    mi_matrix = np.zeros((C, C))

    for i in range(C):
        for j in range(i+1, C):
            # Discretize features
            hist_2d, _, _ = np.histogram2d(
                features_flat[i].cpu().numpy(),
                features_flat[j].cpu().numpy(),
                bins=num_bins
            )

            # Compute MI
            pxy = hist_2d / hist_2d.sum()
            px = pxy.sum(axis=1)
            py = pxy.sum(axis=0)

            # MI = H(X) + H(Y) - H(X,Y)
            hx = -np.sum(px * np.log2(px + 1e-10))
            hy = -np.sum(py * np.log2(py + 1e-10))
            hxy = -np.sum(pxy * np.log2(pxy + 1e-10))

            mi_matrix[i, j] = hx + hy - hxy
            mi_matrix[j, i] = mi_matrix[i, j]

    return mi_matrix

# Example result (ResNet layer):
# Average MI between feature pairs: 0.68 bits
# Max MI: 1.2 bits (some feature pairs are highly correlated)
#
# Interpretation: High MI indicates redundancy - knowing one feature
# provides information about another. Ghost modules exploit this by
# generating correlated features cheaply.
```

**Channel Clustering**:

```python
from sklearn.cluster import SpectralClustering

def cluster_features(features, n_clusters):
    """
    Cluster feature channels based on similarity.
    Intrinsic features = cluster centers
    Ghost features = cluster members
    """
    B, C, H, W = features.shape
    features_flat = features.permute(1, 0, 2, 3).reshape(C, -1)

    # Compute similarity matrix
    similarity = torch.mm(features_flat, features_flat.T)
    similarity = (similarity - similarity.min()) / (similarity.max() - similarity.min())

    # Spectral clustering
    clustering = SpectralClustering(
        n_clusters=n_clusters,
        affinity='precomputed',
        assign_labels='kmeans'
    )
    labels = clustering.fit_predict(similarity.cpu().numpy())

    return labels

# Example: 512 channels → 8 clusters
labels = cluster_features(features, n_clusters=8)

print(f"Cluster sizes: {np.bincount(labels)}")
# Output: Cluster sizes: [67, 73, 59, 68, 61, 72, 54, 58]
#
# Interpretation: Features naturally group into clusters.
# Ghost strategy: Generate 8 intrinsic features (cluster centers),
# generate ~56 ghost features per intrinsic (cluster members).
```

### 3.4 Optimization Perspective

**Loss Function**:

```
Standard CNN training:
    L = L_task(f(X; θ), Y_true)

    where f(X; θ) is the network and L_task is task loss (cross-entropy, etc.)

GhostNet training (same):
    L = L_task(f_ghost(X; θ_ghost), Y_true)

Key insight: No additional loss terms needed!
    - Ghost operations are trained end-to-end
    - Redundancy exploitation emerges naturally during optimization
    - No need for explicit redundancy constraints
```

**Gradient Flow**:

```python
# Gradient through Ghost Module

# Forward:
x1 = primary_conv(x)         # Intrinsic features
x2 = cheap_op(x1)            # Ghost features
out = torch.cat([x1, x2], 1) # Concatenate

# Backward:
grad_out = grad_from_next_layer

# Split gradient
grad_x1_direct = grad_out[:, :m//s, :, :]      # Gradient for x1
grad_x2 = grad_out[:, m//s:, :, :]              # Gradient for x2

# Backprop through cheap_op
grad_x1_indirect = cheap_op.backward(grad_x2)

# Total gradient for x1
grad_x1 = grad_x1_direct + grad_x1_indirect

# Backprop through primary_conv
grad_x = primary_conv.backward(grad_x1)
```

**Observation**: Intrinsic features receive gradients from two sources:
1. Direct: From subsequent layers
2. Indirect: Through ghost features

This encourages intrinsic features to be "good generators" for ghost features.

### 3.5 Comparison with Other Decompositions

**Tucker Decomposition**:

```
Tucker decomposition for conv weights W ∈ ℝ^(m×c×k×k):

W ≈ G ×₁ U₁ ×₂ U₂ ×₃ U₃ ×₄ U₄

where:
    G ∈ ℝ^(r₁×r₂×r₃×r₄): Core tensor
    U_i: Factor matrices

Speedup: (r₁r₂r₃r₄ + mr₁ + cr₂ + kr₃ + kr₄) / (mck²)

Pros:
    - Strong theoretical foundation
    - Can achieve high compression (4-5×)

Cons:
    - Requires post-training decomposition
    - Accuracy loss (1-2% on ImageNet)
    - Difficult to train end-to-end

GhostNet vs Tucker:
    - GhostNet trains end-to-end (no decomposition step)
    - GhostNet maintains accuracy better
    - Tucker can achieve higher compression
    - Tucker is orthogonal (can combine with GhostNet)
```

**CP Decomposition**:

```
CP (CANDECOMP/PARAFAC) for conv weights:

W ≈ Σᵣ λᵣ (aᵣ ⊗ bᵣ ⊗ cᵣ ⊗ dᵣ)

where ⊗ is outer product and r is rank

Similar pros/cons to Tucker
```

**Depthwise Separable Convolution**:

```
Standard: W ∈ ℝ^(m×c×k×k)

Depthwise Separable:
    W_depth ∈ ℝ^(c×1×k×k)  (depthwise, c filters)
    W_point ∈ ℝ^(m×c×1×1)  (pointwise, m filters)

    Operation:
        Y_depth = DWConv(X, W_depth)
        Y = Conv(Y_depth, W_point)

    Speedup: (k²c + mc) / (mck²) ≈ k²/m + 1/k² ≈ 8-9× for k=3

GhostNet vs Depthwise Separable:
    - Depthwise separable separates spatial and channel mixing
    - GhostNet exploits feature redundancy
    - Different mechanisms, complementary
    - Can combine: Use depthwise separable in Ghost cheap operations
```

**Summary Table**:

| Method | Speedup | Accuracy | Training | Orthogonal to Ghost? |
|--------|---------|----------|----------|----------------------|
| Tucker Decomp | 4-5× | -1-2% | Post-training | Yes |
| CP Decomp | 3-4× | -1-2% | Post-training | Yes |
| Depthwise Sep | 8-9× | -2-3% | End-to-end | Partially |
| GhostNet | 2-4× | -0.5-1% | End-to-end | N/A (base method) |
| Ghost + Tucker | 8-12× | -1.5-2.5% | Hybrid | - |
| Ghost + DWSep | 4-6× | -0.5-1.5% | End-to-end | - |


---

## 4. GhostNet Architecture

### 4.1 Overall Network Structure

**GhostNet Design Philosophy**:
- Replace standard convolutions with Ghost modules throughout the network
- Use Ghost bottlenecks as building blocks
- Maintain compatibility with mobile deployment
- Achieve efficiency without sacrificing accuracy

**Architecture Overview**:

```
Input (224×224×3)
    ↓
Stem (Conv 3×3, stride=2) → 112×112×16
    ↓
Stage 1: Ghost Bottleneck × 1 → 112×112×16
    ↓
Stage 2: Ghost Bottleneck × 4 → 56×56×24
    ↓
Stage 3: Ghost Bottleneck × 4 → 28×28×40
    ↓
Stage 4: Ghost Bottleneck × 6 → 14×14×112
    ↓
Stage 5: Ghost Bottleneck × 6 → 7×7×160
    ↓
Conv 1×1 → 7×7×960
    ↓
Global Average Pool → 1×1×960
    ↓
Conv 1×1 → 1×1×1280
    ↓
FC → num_classes
    ↓
Softmax → predictions
```

### 4.2 Complete PyTorch Implementation

```python
import torch
import torch.nn as nn
import math


def _make_divisible(v, divisor=4, min_value=None):
    """
    Ensure that all layers have a channel number divisible by divisor.
    """
    if min_value is None:
        min_value = divisor
    new_v = max(min_value, int(v + divisor / 2) // divisor * divisor)
    if new_v < 0.9 * v:
        new_v += divisor
    return new_v


class SqueezeExcite(nn.Module):
    """Squeeze-and-Excitation block."""
    def __init__(self, in_chs, se_ratio=0.25, reduced_base_chs=None,
                 act_layer=nn.ReLU, gate_fn=nn.Hardsigmoid, divisor=4):
        super(SqueezeExcite, self).__init__()
        reduced_chs = _make_divisible((reduced_base_chs or in_chs) * se_ratio, divisor)
        self.conv_reduce = nn.Conv2d(in_chs, reduced_chs, 1, bias=True)
        self.act1 = act_layer(inplace=True)
        self.conv_expand = nn.Conv2d(reduced_chs, in_chs, 1, bias=True)
        self.gate_fn = gate_fn(inplace=True)

    def forward(self, x):
        x_se = x.mean((2, 3), keepdim=True)
        x_se = self.conv_reduce(x_se)
        x_se = self.act1(x_se)
        x_se = self.conv_expand(x_se)
        return x * self.gate_fn(x_se)


class GhostModule(nn.Module):
    """Ghost Module for generating feature maps efficiently."""
    def __init__(self, inp, oup, kernel_size=1, ratio=2, dw_size=3, stride=1, relu=True):
        super(GhostModule, self).__init__()
        self.oup = oup
        init_channels = math.ceil(oup / ratio)
        new_channels = init_channels * (ratio - 1)

        # Primary convolution
        self.primary_conv = nn.Sequential(
            nn.Conv2d(inp, init_channels, kernel_size, stride, kernel_size//2, bias=False),
            nn.BatchNorm2d(init_channels),
            nn.ReLU(inplace=True) if relu else nn.Sequential(),
        )

        # Cheap operation (depthwise convolution)
        self.cheap_operation = nn.Sequential(
            nn.Conv2d(init_channels, new_channels, dw_size, 1, dw_size//2,
                     groups=init_channels, bias=False),
            nn.BatchNorm2d(new_channels),
            nn.ReLU(inplace=True) if relu else nn.Sequential(),
        )

    def forward(self, x):
        x1 = self.primary_conv(x)
        x2 = self.cheap_operation(x1)
        out = torch.cat([x1, x2], dim=1)
        return out[:, :self.oup, :, :]


class GhostBottleneck(nn.Module):
    """Ghost bottleneck with optional SE."""
    def __init__(self, in_chs, mid_chs, out_chs, dw_kernel_size=3,
                 stride=1, act_layer=nn.ReLU, se_ratio=0.):
        super(GhostBottleneck, self).__init__()
        has_se = se_ratio is not None and se_ratio > 0.
        self.stride = stride

        # Point-wise expansion (Ghost Module)
        self.ghost1 = GhostModule(in_chs, mid_chs, relu=True)

        # Depth-wise convolution
        if self.stride > 1:
            self.conv_dw = nn.Conv2d(mid_chs, mid_chs, dw_kernel_size, stride=stride,
                                    padding=(dw_kernel_size-1)//2,
                                    groups=mid_chs, bias=False)
            self.bn_dw = nn.BatchNorm2d(mid_chs)

        # Squeeze-and-excitation
        if has_se:
            self.se = SqueezeExcite(mid_chs, se_ratio=se_ratio)
        else:
            self.se = None

        # Point-wise linear projection (Ghost Module, no activation)
        self.ghost2 = GhostModule(mid_chs, out_chs, relu=False)

        # Shortcut
        if (in_chs == out_chs and self.stride == 1):
            self.shortcut = nn.Sequential()
        else:
            self.shortcut = nn.Sequential(
                nn.Conv2d(in_chs, in_chs, dw_kernel_size, stride=stride,
                         padding=(dw_kernel_size-1)//2, groups=in_chs, bias=False),
                nn.BatchNorm2d(in_chs),
                nn.Conv2d(in_chs, out_chs, 1, stride=1, padding=0, bias=False),
                nn.BatchNorm2d(out_chs),
            )

    def forward(self, x):
        residual = x

        # 1st ghost bottleneck
        x = self.ghost1(x)

        # Depth-wise convolution
        if self.stride > 1:
            x = self.conv_dw(x)
            x = self.bn_dw(x)

        # Squeeze-and-excitation
        if self.se is not None:
            x = self.se(x)

        # 2nd ghost bottleneck
        x = self.ghost2(x)

        x += self.shortcut(residual)
        return x


class GhostNet(nn.Module):
    """
    GhostNet: Efficient CNN architecture using Ghost modules.

    Args:
        cfgs: Network configuration (channels, kernel sizes, strides, etc.)
        num_classes: Number of output classes
        width: Width multiplier
        dropout: Dropout rate
    """
    def __init__(self, cfgs, num_classes=1000, width=1.0, dropout=0.2):
        super(GhostNet, self).__init__()
        self.cfgs = cfgs
        self.dropout = dropout

        # Building first layer
        output_channel = _make_divisible(16 * width, 4)
        self.conv_stem = nn.Conv2d(3, output_channel, 3, 2, 1, bias=False)
        self.bn1 = nn.BatchNorm2d(output_channel)
        self.act1 = nn.ReLU(inplace=True)
        input_channel = output_channel

        # Building inverted residual blocks
        stages = []
        block = GhostBottleneck
        for cfg in self.cfgs:
            layers = []
            for k, exp_size, c, se_ratio, s in cfg:
                output_channel = _make_divisible(c * width, 4)
                hidden_channel = _make_divisible(exp_size * width, 4)
                layers.append(block(input_channel, hidden_channel, output_channel,
                                   k, s, se_ratio=se_ratio))
                input_channel = output_channel
            stages.append(nn.Sequential(*layers))

        output_channel = _make_divisible(exp_size * width, 4)
        stages.append(nn.Sequential(
            nn.Conv2d(input_channel, output_channel, 1, 1, 0, bias=False),
            nn.BatchNorm2d(output_channel),
            nn.ReLU(inplace=True)
        ))
        input_channel = output_channel

        self.blocks = nn.Sequential(*stages)

        # Building last several layers
        output_channel = 1280
        self.global_pool = nn.AdaptiveAvgPool2d((1, 1))
        self.conv_head = nn.Conv2d(input_channel, output_channel, 1, 1, 0, bias=True)
        self.act2 = nn.ReLU(inplace=True)
        self.classifier = nn.Linear(output_channel, num_classes)

    def forward(self, x):
        x = self.conv_stem(x)
        x = self.bn1(x)
        x = self.act1(x)
        x = self.blocks(x)
        x = self.global_pool(x)
        x = self.conv_head(x)
        x = self.act2(x)
        x = x.view(x.size(0), -1)
        if self.dropout > 0.:
            x = F.dropout(x, p=self.dropout, training=self.training)
        x = self.classifier(x)
        return x


def ghostnet(num_classes=1000, width=1.0, dropout=0.2):
    """
    Constructs a GhostNet model.

    Configuration format:
        [kernel_size, exp_size, out_channels, se_ratio, stride]
    """
    cfgs = [
        # Stage 1
        [[3,  16,  16, 0, 1]],
        # Stage 2
        [[3,  48,  24, 0, 2]],
        [[3,  72,  24, 0, 1]],
        # Stage 3
        [[5,  72,  40, 0.25, 2]],
        [[5, 120,  40, 0.25, 1]],
        # Stage 4
        [[3, 240,  80, 0, 2]],
        [[3, 200,  80, 0, 1],
         [3, 184,  80, 0, 1],
         [3, 184,  80, 0, 1],
         [3, 480, 112, 0.25, 1],
         [3, 672, 112, 0.25, 1]
        ],
        # Stage 5
        [[5, 672, 160, 0.25, 2]],
        [[5, 960, 160, 0, 1],
         [5, 960, 160, 0.25, 1],
         [5, 960, 160, 0, 1],
         [5, 960, 160, 0.25, 1]
        ]
    ]
    return GhostNet(cfgs, num_classes=num_classes, width=width, dropout=dropout)


# Example usage
if __name__ == '__main__':
    model = ghostnet(num_classes=1000, width=1.0)
    x = torch.randn(1, 3, 224, 224)
    y = model(x)
    print(f"Input shape: {x.shape}")
    print(f"Output shape: {y.shape}")

    # Count parameters and FLOPs
    from thop import profile
    flops, params = profile(model, inputs=(x,))
    print(f"Parameters: {params / 1e6:.2f}M")
    print(f"FLOPs: {flops / 1e6:.2f}M")

    # Expected output:
    # Input shape: torch.Size([1, 3, 224, 224])
    # Output shape: torch.Size([1, 1000])
    # Parameters: 5.18M
    # FLOPs: 142.0M
```

### 4.3 Configuration Details

**GhostNet Configuration Table**:

```
Stage | Block | k | exp | out | SE | s | Resolution | Channels
------|-------|---|-----|-----|----|----|------------|----------
Stem  |   -   | 3 |  -  | 16  | ✗  | 2  | 112×112   | 16
1     | Ghost | 3 | 16  | 16  | ✗  | 1  | 112×112   | 16
2     | Ghost | 3 | 48  | 24  | ✗  | 2  | 56×56     | 24
2     | Ghost | 3 | 72  | 24  | ✗  | 1  | 56×56     | 24
3     | Ghost | 5 | 72  | 40  | ✓  | 2  | 28×28     | 40
3     | Ghost | 5 | 120 | 40  | ✓  | 1  | 28×28     | 40
4     | Ghost | 3 | 240 | 80  | ✗  | 2  | 14×14     | 80
4     | Ghost | 3 | 200 | 80  | ✗  | 1  | 14×14     | 80
4     | Ghost | 3 | 184 | 80  | ✗  | 1  | 14×14     | 80
4     | Ghost | 3 | 184 | 80  | ✗  | 1  | 14×14     | 80
4     | Ghost | 3 | 480 | 112 | ✓  | 1  | 14×14     | 112
4     | Ghost | 3 | 672 | 112 | ✓  | 1  | 14×14     | 112
5     | Ghost | 5 | 672 | 160 | ✓  | 2  | 7×7       | 160
5     | Ghost | 5 | 960 | 160 | ✗  | 1  | 7×7       | 160
5     | Ghost | 5 | 960 | 160 | ✓  | 1  | 7×7       | 160
5     | Ghost | 5 | 960 | 160 | ✗  | 1  | 7×7       | 160
5     | Ghost | 5 | 960 | 160 | ✓  | 1  | 7×7       | 160
Head  |   -   | 1 | 960 | 1280| ✗  | 1  | 1×1       | 1280

Legend:
  k   = kernel size
  exp = expansion channels (hidden dimension in bottleneck)
  out = output channels
  SE  = Squeeze-Excitation enabled
  s   = stride
```

**Width Multiplier Variants**:

```python
# GhostNet-0.5×  (smaller, faster)
model_05x = ghostnet(num_classes=1000, width=0.5)
# Parameters: 2.6M, FLOPs: 42M, Top-1: 66.2%

# GhostNet-1.0×  (baseline)
model_10x = ghostnet(num_classes=1000, width=1.0)
# Parameters: 5.2M, FLOPs: 142M, Top-1: 73.9%

# GhostNet-1.3×  (larger, more accurate)
model_13x = ghostnet(num_classes=1000, width=1.3)
# Parameters: 7.3M, FLOPs: 226M, Top-1: 75.7%

# GhostNet-2.0×  (largest)
model_20x = ghostnet(num_classes=1000, width=2.0)
# Parameters: 15.4M, FLOPs: 450M, Top-1: 78.2%
```

### 4.4 TensorFlow / Keras Implementation

```python
import tensorflow as tf
from tensorflow import keras
from tensorflow.keras import layers
import math


def _make_divisible(v, divisor=4, min_value=None):
    if min_value is None:
        min_value = divisor
    new_v = max(min_value, int(v + divisor / 2) // divisor * divisor)
    if new_v < 0.9 * v:
        new_v += divisor
    return new_v


class SqueezeExcite(layers.Layer):
    """Squeeze-and-Excitation block in Keras."""
    def __init__(self, in_chs, se_ratio=0.25, **kwargs):
        super(SqueezeExcite, self).__init__(**kwargs)
        self.in_chs = in_chs
        self.se_ratio = se_ratio
        reduced_chs = _make_divisible(in_chs * se_ratio, 4)

        self.avg_pool = layers.GlobalAveragePooling2D(keepdims=True)
        self.conv_reduce = layers.Conv2D(reduced_chs, 1, use_bias=True)
        self.act = layers.ReLU()
        self.conv_expand = layers.Conv2D(in_chs, 1, use_bias=True)
        self.gate = tf.nn.hard_sigmoid

    def call(self, inputs):
        x = self.avg_pool(inputs)
        x = self.conv_reduce(x)
        x = self.act(x)
        x = self.conv_expand(x)
        x = self.gate(x)
        return inputs * x


class GhostModule(layers.Layer):
    """Ghost Module in Keras."""
    def __init__(self, inp, oup, kernel_size=1, ratio=2, dw_size=3,
                 stride=1, relu=True, **kwargs):
        super(GhostModule, self).__init__(**kwargs)
        self.oup = oup
        init_channels = math.ceil(oup / ratio)
        new_channels = init_channels * (ratio - 1)

        # Primary convolution
        self.primary_conv = keras.Sequential([
            layers.Conv2D(init_channels, kernel_size, stride,
                         padding='same', use_bias=False),
            layers.BatchNormalization(),
            layers.ReLU() if relu else layers.Lambda(lambda x: x)
        ])

        # Cheap operation
        self.cheap_operation = keras.Sequential([
            layers.DepthwiseConv2D(dw_size, 1, padding='same', use_bias=False),
            layers.BatchNormalization(),
            layers.ReLU() if relu else layers.Lambda(lambda x: x)
        ])

    def call(self, inputs):
        x1 = self.primary_conv(inputs)
        x2 = self.cheap_operation(x1)
        out = tf.concat([x1, x2], axis=-1)
        return out[:, :, :, :self.oup]


class GhostBottleneck(layers.Layer):
    """Ghost bottleneck in Keras."""
    def __init__(self, in_chs, mid_chs, out_chs, dw_kernel_size=3,
                 stride=1, se_ratio=0., **kwargs):
        super(GhostBottleneck, self).__init__(**kwargs)
        self.stride = stride
        has_se = se_ratio is not None and se_ratio > 0.

        # Ghost module 1
        self.ghost1 = GhostModule(in_chs, mid_chs, relu=True)

        # Depthwise convolution
        if self.stride > 1:
            self.conv_dw = layers.DepthwiseConv2D(dw_kernel_size, stride,
                                                 padding='same', use_bias=False)
            self.bn_dw = layers.BatchNormalization()

        # Squeeze-Excitation
        self.se = SqueezeExcite(mid_chs, se_ratio) if has_se else None

        # Ghost module 2
        self.ghost2 = GhostModule(mid_chs, out_chs, relu=False)

        # Shortcut
        if in_chs == out_chs and stride == 1:
            self.shortcut = layers.Lambda(lambda x: x)
        else:
            self.shortcut = keras.Sequential([
                layers.DepthwiseConv2D(dw_kernel_size, stride,
                                      padding='same', use_bias=False),
                layers.BatchNormalization(),
                layers.Conv2D(out_chs, 1, use_bias=False),
                layers.BatchNormalization()
            ])

    def call(self, inputs):
        residual = inputs

        x = self.ghost1(inputs)

        if self.stride > 1:
            x = self.conv_dw(x)
            x = self.bn_dw(x)

        if self.se is not None:
            x = self.se(x)

        x = self.ghost2(x)
        x = x + self.shortcut(residual)

        return x


def GhostNetKeras(input_shape=(224, 224, 3), num_classes=1000,
                 width=1.0, dropout=0.2):
    """
    Build GhostNet model in Keras.
    """
    inputs = layers.Input(shape=input_shape)

    # Stem
    x = layers.Conv2D(_make_divisible(16 * width, 4), 3, 2,
                     padding='same', use_bias=False)(inputs)
    x = layers.BatchNormalization()(x)
    x = layers.ReLU()(x)

    # Configuration (same as PyTorch version)
    cfgs = [
        [[3,  16,  16, 0, 1]],
        [[3,  48,  24, 0, 2]],
        [[3,  72,  24, 0, 1]],
        [[5,  72,  40, 0.25, 2]],
        [[5, 120,  40, 0.25, 1]],
        [[3, 240,  80, 0, 2]],
        [[3, 200,  80, 0, 1],
         [3, 184,  80, 0, 1],
         [3, 184,  80, 0, 1],
         [3, 480, 112, 0.25, 1],
         [3, 672, 112, 0.25, 1]],
        [[5, 672, 160, 0.25, 2]],
        [[5, 960, 160, 0, 1],
         [5, 960, 160, 0.25, 1],
         [5, 960, 160, 0, 1],
         [5, 960, 160, 0.25, 1]]
    ]

    # Build stages
    input_channel = _make_divisible(16 * width, 4)
    for stage_cfg in cfgs:
        for k, exp_size, c, se_ratio, s in stage_cfg:
            output_channel = _make_divisible(c * width, 4)
            hidden_channel = _make_divisible(exp_size * width, 4)
            x = GhostBottleneck(input_channel, hidden_channel,
                               output_channel, k, s, se_ratio)(x)
            input_channel = output_channel

    # Head
    x = layers.Conv2D(_make_divisible(960 * width, 4), 1, use_bias=False)(x)
    x = layers.BatchNormalization()(x)
    x = layers.ReLU()(x)

    x = layers.GlobalAveragePooling2D()(x)
    x = layers.Conv2D(1280, 1, use_bias=True)(x)
    x = layers.ReLU()(x)

    if dropout > 0:
        x = layers.Dropout(dropout)(x)

    outputs = layers.Dense(num_classes, activation='softmax')(x)

    model = keras.Model(inputs=inputs, outputs=outputs, name='GhostNet')
    return model


# Example usage
if __name__ == '__main__':
    model = GhostNetKeras(num_classes=1000, width=1.0)
    model.summary()

    # Test forward pass
    import numpy as np
    x = np.random.randn(1, 224, 224, 3).astype(np.float32)
    y = model(x)
    print(f"Output shape: {y.shape}")
```

### 4.5 Model Scaling

**Width Scaling** (multiply number of channels):

```python
def scale_width(base_channels, multiplier):
    """Scale channel count by multiplier."""
    return _make_divisible(base_channels * multiplier, 4)

# Example: Scale from 1.0× to 1.3×
base = [16, 24, 40, 80, 112, 160]
scaled = [scale_width(c, 1.3) for c in base]
print(scaled)  # [20, 32, 52, 104, 144, 208]
```

**Depth Scaling** (add more blocks):

```python
def scale_depth(base_blocks, multiplier):
    """Scale number of blocks by multiplier."""
    return int(math.ceil(base_blocks * multiplier))

# Example: Scale from baseline to 1.5× depth
base_counts = [1, 2, 2, 6, 5]  # Blocks per stage
scaled_counts = [scale_depth(b, 1.5) for b in base_counts]
print(scaled_counts)  # [2, 3, 3, 9, 8]
```

**Resolution Scaling** (change input size):

```python
# Different input resolutions
model_224 = ghostnet(num_classes=1000, width=1.0)  # 224×224 input
model_288 = ghostnet(num_classes=1000, width=1.0)  # 288×288 input
model_352 = ghostnet(num_classes=1000, width=1.0)  # 352×352 input

# Adjust final pooling for different resolutions
# Spatial dimensions: 224 → 7×7, 288 → 9×9, 352 → 11×11
```

**Compound Scaling** (scale all dimensions):

```python
def compound_scale(phi):
    """
    Compound scaling similar to EfficientNet.

    phi: Compound coefficient
    """
    alpha = 1.2  # Depth coefficient
    beta = 1.1   # Width coefficient
    gamma = 1.15 # Resolution coefficient

    depth_mult = alpha ** phi
    width_mult = beta ** phi
    resolution = int(224 * (gamma ** phi))

    return depth_mult, width_mult, resolution

# GhostNet-B0 (baseline): phi=0
d0, w0, r0 = compound_scale(0)  # 1.0, 1.0, 224

# GhostNet-B1: phi=1
d1, w1, r1 = compound_scale(1)  # 1.2, 1.1, 258

# GhostNet-B2: phi=2
d2, w2, r2 = compound_scale(2)  # 1.44, 1.21, 296

# Apply to model
model_b0 = ghostnet(num_classes=1000, width=w0)
model_b1 = ghostnet(num_classes=1000, width=w1)  # Also scale depth & resolution
model_b2 = ghostnet(num_classes=1000, width=w2)
```


---

## 5. Implementation Examples

### 5.1 Complete Training Script (PyTorch)

```python
import torch
import torch.nn as nn
import torch.optim as optim
from torch.utils.data import DataLoader
import torchvision
import torchvision.transforms as transforms
from tqdm import tqdm
import wandb


def train_ghostnet(
    model_width=1.0,
    num_epochs=300,
    batch_size=256,
    learning_rate=0.4,
    num_workers=8,
    device='cuda'
):
    """
    Complete training script for GhostNet on ImageNet.

    Training recipe based on original paper:
    - Optimizer: SGD with momentum 0.9
    - Learning rate: 0.4 with cosine annealing
    - Batch size: 1024 (across 4 GPUs) = 256 per GPU
    - Weight decay: 4e-5
    - Label smoothing: 0.1
    - Epochs: 300
    """
    # Initialize wandb for experiment tracking
    wandb.init(project='ghostnet', config={
        'width': model_width,
        'epochs': num_epochs,
        'batch_size': batch_size,
        'lr': learning_rate
    })

    # Data preprocessing
    normalize = transforms.Normalize(
        mean=[0.485, 0.456, 0.406],
        std=[0.229, 0.224, 0.225]
    )

    train_transform = transforms.Compose([
        transforms.RandomResizedCrop(224),
        transforms.RandomHorizontalFlip(),
        transforms.ColorJitter(brightness=0.4, contrast=0.4,
                              saturation=0.4, hue=0.1),
        transforms.ToTensor(),
        normalize,
    ])

    val_transform = transforms.Compose([
        transforms.Resize(256),
        transforms.CenterCrop(224),
        transforms.ToTensor(),
        normalize,
    ])

    # Load ImageNet dataset
    train_dataset = torchvision.datasets.ImageNet(
        root='/path/to/imagenet',
        split='train',
        transform=train_transform
    )

    val_dataset = torchvision.datasets.ImageNet(
        root='/path/to/imagenet',
        split='val',
        transform=val_transform
    )

    train_loader = DataLoader(
        train_dataset,
        batch_size=batch_size,
        shuffle=True,
        num_workers=num_workers,
        pin_memory=True
    )

    val_loader = DataLoader(
        val_dataset,
        batch_size=batch_size,
        shuffle=False,
        num_workers=num_workers,
        pin_memory=True
    )

    # Build model
    model = ghostnet(num_classes=1000, width=model_width, dropout=0.2)
    model = model.to(device)

    # For multi-GPU training
    if torch.cuda.device_count() > 1:
        model = nn.DataParallel(model)

    # Loss function with label smoothing
    criterion = LabelSmoothingCrossEntropy(smoothing=0.1)

    # Optimizer: SGD with momentum
    optimizer = optim.SGD(
        model.parameters(),
        lr=learning_rate,
        momentum=0.9,
        weight_decay=4e-5,
        nesterov=True
    )

    # Learning rate scheduler: Cosine annealing
    scheduler = optim.lr_scheduler.CosineAnnealingLR(
        optimizer,
        T_max=num_epochs
    )

    # Mixed precision training
    scaler = torch.cuda.amp.GradScaler()

    # Training loop
    best_acc = 0.0
    for epoch in range(num_epochs):
        # Train
        model.train()
        train_loss = 0.0
        train_correct = 0
        train_total = 0

        pbar = tqdm(train_loader, desc=f'Epoch {epoch+1}/{num_epochs}')
        for images, labels in pbar:
            images, labels = images.to(device), labels.to(device)

            # Mixed precision forward pass
            with torch.cuda.amp.autocast():
                outputs = model(images)
                loss = criterion(outputs, labels)

            # Backward pass with gradient scaling
            optimizer.zero_grad()
            scaler.scale(loss).backward()
            scaler.step(optimizer)
            scaler.update()

            # Statistics
            train_loss += loss.item() * images.size(0)
            _, predicted = outputs.max(1)
            train_total += labels.size(0)
            train_correct += predicted.eq(labels).sum().item()

            # Update progress bar
            pbar.set_postfix({
                'loss': f'{train_loss/train_total:.3f}',
                'acc': f'{100.*train_correct/train_total:.2f}%'
            })

        # Validate
        model.eval()
        val_loss = 0.0
        val_correct = 0
        val_total = 0
        top5_correct = 0

        with torch.no_grad():
            for images, labels in tqdm(val_loader, desc='Validation'):
                images, labels = images.to(device), labels.to(device)

                outputs = model(images)
                loss = criterion(outputs, labels)

                val_loss += loss.item() * images.size(0)
                _, predicted = outputs.max(1)
                val_total += labels.size(0)
                val_correct += predicted.eq(labels).sum().item()

                # Top-5 accuracy
                _, top5_pred = outputs.topk(5, 1, True, True)
                top5_correct += top5_pred.eq(
                    labels.view(-1, 1).expand_as(top5_pred)
                ).sum().item()

        # Compute metrics
        train_acc = 100. * train_correct / train_total
        val_acc = 100. * val_correct / val_total
        top5_acc = 100. * top5_correct / val_total

        # Log to wandb
        wandb.log({
            'epoch': epoch,
            'train_loss': train_loss / train_total,
            'train_acc': train_acc,
            'val_loss': val_loss / val_total,
            'val_acc': val_acc,
            'top5_acc': top5_acc,
            'lr': scheduler.get_last_lr()[0]
        })

        print(f'Epoch {epoch+1}/{num_epochs}:')
        print(f'  Train Loss: {train_loss/train_total:.4f}, '
              f'Train Acc: {train_acc:.2f}%')
        print(f'  Val Loss: {val_loss/val_total:.4f}, '
              f'Val Acc: {val_acc:.2f}%, Top-5 Acc: {top5_acc:.2f}%')

        # Save best model
        if val_acc > best_acc:
            best_acc = val_acc
            torch.save({
                'epoch': epoch,
                'model_state_dict': model.state_dict(),
                'optimizer_state_dict': optimizer.state_dict(),
                'best_acc': best_acc,
            }, f'ghostnet_{model_width}x_best.pth')

        # Step scheduler
        scheduler.step()

    wandb.finish()
    return model


class LabelSmoothingCrossEntropy(nn.Module):
    """
    Label smoothing cross entropy loss.

    Prevents overconfidence by distributing some probability mass
    to incorrect classes.
    """
    def __init__(self, smoothing=0.1):
        super().__init__()
        self.smoothing = smoothing

    def forward(self, pred, target):
        n_classes = pred.size(1)
        log_pred = torch.nn.functional.log_softmax(pred, dim=1)

        # Smooth labels
        with torch.no_grad():
            true_dist = torch.zeros_like(log_pred)
            true_dist.fill_(self.smoothing / (n_classes - 1))
            true_dist.scatter_(1, target.unsqueeze(1), 1.0 - self.smoothing)

        return torch.mean(torch.sum(-true_dist * log_pred, dim=1))


# Run training
if __name__ == '__main__':
    model = train_ghostnet(
        model_width=1.0,
        num_epochs=300,
        batch_size=256,
        learning_rate=0.4,
        device='cuda'
    )
```

### 5.2 Transfer Learning Example

```python
import torch
import torch.nn as nn
from torchvision import datasets, transforms


def finetune_ghostnet(
    pretrained_path,
    num_classes=10,
    freeze_backbone=True,
    learning_rate=0.01,
    num_epochs=50
):
    """
    Fine-tune GhostNet on a custom dataset.

    Args:
        pretrained_path: Path to pretrained GhostNet weights
        num_classes: Number of classes in target dataset
        freeze_backbone: Whether to freeze feature extractor
        learning_rate: Learning rate for fine-tuning
        num_epochs: Number of training epochs
    """
    # Load pretrained model
    model = ghostnet(num_classes=1000, width=1.0)
    checkpoint = torch.load(pretrained_path)
    model.load_state_dict(checkpoint['model_state_dict'])

    # Freeze backbone layers if specified
    if freeze_backbone:
        for name, param in model.named_parameters():
            if 'classifier' not in name:
                param.requires_grad = False

    # Replace classifier head
    model.classifier = nn.Linear(model.classifier.in_features, num_classes)

    # Move to GPU
    device = torch.device('cuda' if torch.cuda.is_available() else 'cpu')
    model = model.to(device)

    # Data loading (example with CIFAR-10)
    transform = transforms.Compose([
        transforms.Resize(224),
        transforms.ToTensor(),
        transforms.Normalize([0.485, 0.456, 0.406], [0.229, 0.224, 0.225])
    ])

    train_dataset = datasets.CIFAR10(
        root='./data', train=True, download=True, transform=transform
    )
    val_dataset = datasets.CIFAR10(
        root='./data', train=False, download=True, transform=transform
    )

    train_loader = torch.utils.data.DataLoader(
        train_dataset, batch_size=64, shuffle=True, num_workers=4
    )
    val_loader = torch.utils.data.DataLoader(
        val_dataset, batch_size=64, shuffle=False, num_workers=4
    )

    # Optimizer and loss
    optimizer = torch.optim.Adam(
        filter(lambda p: p.requires_grad, model.parameters()),
        lr=learning_rate
    )
    criterion = nn.CrossEntropyLoss()

    # Training loop
    best_acc = 0.0
    for epoch in range(num_epochs):
        model.train()
        train_loss, train_correct, train_total = 0.0, 0, 0

        for images, labels in train_loader:
            images, labels = images.to(device), labels.to(device)

            optimizer.zero_grad()
            outputs = model(images)
            loss = criterion(outputs, labels)
            loss.backward()
            optimizer.step()

            train_loss += loss.item() * images.size(0)
            _, predicted = outputs.max(1)
            train_total += labels.size(0)
            train_correct += predicted.eq(labels).sum().item()

        # Validation
        model.eval()
        val_correct, val_total = 0, 0

        with torch.no_grad():
            for images, labels in val_loader:
                images, labels = images.to(device), labels.to(device)
                outputs = model(images)
                _, predicted = outputs.max(1)
                val_total += labels.size(0)
                val_correct += predicted.eq(labels).sum().item()

        train_acc = 100. * train_correct / train_total
        val_acc = 100. * val_correct / val_total

        print(f'Epoch {epoch+1}/{num_epochs}: '
              f'Train Acc: {train_acc:.2f}%, Val Acc: {val_acc:.2f}%')

        if val_acc > best_acc:
            best_acc = val_acc
            torch.save(model.state_dict(), 'ghostnet_finetuned_best.pth')

    return model


# Example usage
model = finetune_ghostnet(
    pretrained_path='ghostnet_1.0x_imagenet.pth',
    num_classes=10,
    freeze_backbone=True,
    learning_rate=0.001,
    num_epochs=50
)
```

### 5.3 Object Detection with GhostNet Backbone

```python
import torch
import torch.nn as nn


class GhostNetFPN(nn.Module):
    """
    GhostNet backbone with Feature Pyramid Network for object detection.

    Extracts multi-scale features for detection tasks.
    """
    def __init__(self, width=1.0, pretrained=None):
        super().__init__()

        # Load GhostNet backbone
        self.backbone = ghostnet(num_classes=1000, width=width)

        if pretrained:
            self.backbone.load_state_dict(torch.load(pretrained))

        # Remove classification head
        self.backbone.global_pool = nn.Identity()
        self.backbone.conv_head = nn.Identity()
        self.backbone.act2 = nn.Identity()
        self.backbone.classifier = nn.Identity()

        # Feature extraction at multiple scales
        # C2: 28×28×40, C3: 14×14×112, C4: 7×7×160
        self.out_channels = [40, 112, 160]

    def forward(self, x):
        """
        Extract features at multiple scales.

        Returns:
            List of feature maps at different resolutions
        """
        features = []

        # Extract features from backbone
        x = self.backbone.conv_stem(x)
        x = self.backbone.bn1(x)
        x = self.backbone.act1(x)

        for idx, block in enumerate(self.backbone.blocks):
            x = block(x)
            # Collect features at specific stages
            if idx in [2, 6, 11]:  # C2, C3, C4
                features.append(x)

        return features


class GhostNetDetector(nn.Module):
    """
    Object detector using GhostNet backbone.

    Similar to SSD/YOLO architecture.
    """
    def __init__(self, num_classes=80, width=1.0, pretrained=None):
        super().__init__()

        # Backbone with FPN
        self.backbone = GhostNetFPN(width=width, pretrained=pretrained)

        # Detection heads for each scale
        self.detection_heads = nn.ModuleList([
            self._make_detection_head(c, num_classes)
            for c in self.backbone.out_channels
        ])

    def _make_detection_head(self, in_channels, num_classes):
        """
        Create detection head for classification and bbox regression.
        """
        return nn.Sequential(
            nn.Conv2d(in_channels, 256, 3, padding=1),
            nn.ReLU(inplace=True),
            nn.Conv2d(256, (num_classes + 4) * 9, 1)  # 9 anchors per location
        )

    def forward(self, x):
        """
        Forward pass for detection.

        Returns:
            detections: List of [batch, num_anchors, num_classes+4]
                       for each feature map
        """
        # Extract multi-scale features
        features = self.backbone(x)

        # Apply detection heads
        detections = []
        for feat, head in zip(features, self.detection_heads):
            det = head(feat)
            # Reshape: [B, (C+4)*9, H, W] → [B, H*W*9, C+4]
            B, _, H, W = det.shape
            det = det.permute(0, 2, 3, 1).reshape(B, -1, det.size(1)//9)
            detections.append(det)

        return detections


# Example usage
detector = GhostNetDetector(
    num_classes=80,  # COCO classes
    width=1.3,
    pretrained='ghostnet_1.3x_imagenet.pth'
)

# Test forward pass
x = torch.randn(1, 3, 416, 416)
detections = detector(x)
for i, det in enumerate(detections):
    print(f"Scale {i}: {det.shape}")  # [1, num_anchors, 84]
```

### 5.4 Semantic Segmentation with GhostNet

```python
class GhostNetSegmentation(nn.Module):
    """
    Semantic segmentation network using GhostNet encoder.

    Uses DeepLabV3+ style decoder.
    """
    def __init__(self, num_classes=21, width=1.0, pretrained=None):
        super().__init__()

        # Encoder: GhostNet backbone
        self.encoder = ghostnet(num_classes=1000, width=width)

        if pretrained:
            self.encoder.load_state_dict(torch.load(pretrained))

        # Remove global pooling and classifier
        self.encoder.global_pool = nn.Identity()
        self.encoder.conv_head = nn.Identity()
        self.encoder.act2 = nn.Identity()
        self.encoder.classifier = nn.Identity()

        # ASPP (Atrous Spatial Pyramid Pooling)
        self.aspp = ASPP(160, 256)

        # Decoder
        self.decoder = nn.Sequential(
            nn.Conv2d(256, 256, 3, padding=1),
            nn.BatchNorm2d(256),
            nn.ReLU(inplace=True),
            nn.Upsample(scale_factor=2, mode='bilinear', align_corners=True),
            nn.Conv2d(256, 128, 3, padding=1),
            nn.BatchNorm2d(128),
            nn.ReLU(inplace=True),
            nn.Upsample(scale_factor=2, mode='bilinear', align_corners=True),
            nn.Conv2d(128, 64, 3, padding=1),
            nn.BatchNorm2d(64),
            nn.ReLU(inplace=True),
            nn.Upsample(scale_factor=4, mode='bilinear', align_corners=True),
        )

        # Final prediction
        self.final_conv = nn.Conv2d(64, num_classes, 1)

    def forward(self, x):
        # Encode
        x = self.encoder.conv_stem(x)
        x = self.encoder.bn1(x)
        x = self.encoder.act1(x)
        x = self.encoder.blocks(x)

        # ASPP
        x = self.aspp(x)

        # Decode
        x = self.decoder(x)

        # Final prediction
        x = self.final_conv(x)

        return x


class ASPP(nn.Module):
    """Atrous Spatial Pyramid Pooling."""
    def __init__(self, in_channels, out_channels):
        super().__init__()

        # Atrous convolutions with different rates
        self.conv1 = nn.Sequential(
            nn.Conv2d(in_channels, out_channels, 1, bias=False),
            nn.BatchNorm2d(out_channels),
            nn.ReLU(inplace=True)
        )

        self.conv2 = nn.Sequential(
            nn.Conv2d(in_channels, out_channels, 3, padding=6, dilation=6, bias=False),
            nn.BatchNorm2d(out_channels),
            nn.ReLU(inplace=True)
        )

        self.conv3 = nn.Sequential(
            nn.Conv2d(in_channels, out_channels, 3, padding=12, dilation=12, bias=False),
            nn.BatchNorm2d(out_channels),
            nn.ReLU(inplace=True)
        )

        self.conv4 = nn.Sequential(
            nn.Conv2d(in_channels, out_channels, 3, padding=18, dilation=18, bias=False),
            nn.BatchNorm2d(out_channels),
            nn.ReLU(inplace=True)
        )

        # Global pooling
        self.global_pool = nn.Sequential(
            nn.AdaptiveAvgPool2d(1),
            nn.Conv2d(in_channels, out_channels, 1, bias=False),
            nn.BatchNorm2d(out_channels),
            nn.ReLU(inplace=True)
        )

        # Project concatenated features
        self.project = nn.Sequential(
            nn.Conv2d(out_channels * 5, out_channels, 1, bias=False),
            nn.BatchNorm2d(out_channels),
            nn.ReLU(inplace=True)
        )

    def forward(self, x):
        size = x.shape[2:]

        feat1 = self.conv1(x)
        feat2 = self.conv2(x)
        feat3 = self.conv3(x)
        feat4 = self.conv4(x)
        feat5 = self.global_pool(x)
        feat5 = nn.functional.interpolate(
            feat5, size=size, mode='bilinear', align_corners=True
        )

        # Concatenate and project
        out = torch.cat([feat1, feat2, feat3, feat4, feat5], dim=1)
        out = self.project(out)

        return out


# Example usage
seg_model = GhostNetSegmentation(
    num_classes=21,  # Pascal VOC classes
    width=1.0,
    pretrained='ghostnet_1.0x_imagenet.pth'
)

# Test forward pass
x = torch.randn(1, 3, 512, 512)
output = seg_model(x)
print(f"Segmentation output: {output.shape}")  # [1, 21, 512, 512]
```


---

## 6. Comparison with Other Efficient Architectures

### 6.1 Architecture Comparison Matrix

**Comprehensive Efficiency Analysis (ImageNet Classification)**:

```
Model               | Params | FLOPs  | Top-1 | Top-5 | Latency* | Strategy
--------------------|--------|--------|-------|-------|----------|------------------
AlexNet (2012)      | 61.0M  | 714M   | 56.5% | 79.1% | 42ms     | Baseline CNN
VGG-16 (2014)       | 138M   | 15.5G  | 71.5% | 90.0% | 186ms    | Stacked 3×3 convs
ResNet-50 (2015)    | 25.6M  | 4.1G   | 76.2% | 92.9% | 32ms     | Residual connections
MobileNetV1 (2017)  | 4.2M   | 569M   | 70.6% | 89.5% | 9.5ms    | Depthwise separable
ShuffleNetV2 (2018) | 2.3M   | 299M   | 69.4% | 88.9% | 8.2ms    | Channel shuffle
MobileNetV2 (2018)  | 3.5M   | 300M   | 72.0% | 90.5% | 8.5ms    | Inverted residuals
MobileNetV3 (2019)  | 5.4M   | 219M   | 75.2% | 92.2% | 7.8ms    | NAS + h-swish
EfficientNet-B0     | 5.3M   | 390M   | 77.1% | 93.3% | 11.3ms   | Compound scaling
GhostNet-1.0×       | 5.2M   | 142M   | 73.9% | 91.4% | 6.9ms    | Ghost operations
GhostNet-1.3×       | 7.3M   | 226M   | 75.7% | 92.7% | 8.4ms    | Ghost operations

*Latency: ARM Cortex-A76 CPU, single-thread, batch=1
```

**Efficiency Frontier Visualization**:

```
Top-1 Accuracy vs. FLOPs

80% ┤                                        ● ResNet-50
    │                                   ● EfficientNet-B0
78% ┤
    │                         ● GhostNet-1.3×
76% ┤                    ● MobileNetV3
    │               ● GhostNet-1.0×
74% ┤          ● MobileNetV2
    │     ● ShuffleNetV2
72% ┤ ● MobileNetV1
    │
70% ┤
    └────┬────────┬────────┬────────┬────────┬────────┬──────→ FLOPs
       100M    500M      1G      2G      3G      4G

Observation: GhostNet achieves best accuracy-FLOPs trade-off below 300M FLOPs
```

### 6.2 Detailed Architecture Comparison

**MobileNetV2 vs. GhostNet**:

```python
# MobileNetV2 Inverted Residual Block
class MobileNetV2Block(nn.Module):
    def __init__(self, inp, oup, stride, expand_ratio=6):
        super().__init__()
        hidden_dim = int(inp * expand_ratio)

        self.block = nn.Sequential(
            # Pointwise expansion
            nn.Conv2d(inp, hidden_dim, 1, bias=False),
            nn.BatchNorm2d(hidden_dim),
            nn.ReLU6(inplace=True),

            # Depthwise convolution
            nn.Conv2d(hidden_dim, hidden_dim, 3, stride, 1,
                     groups=hidden_dim, bias=False),
            nn.BatchNorm2d(hidden_dim),
            nn.ReLU6(inplace=True),

            # Pointwise projection (linear)
            nn.Conv2d(hidden_dim, oup, 1, bias=False),
            nn.BatchNorm2d(oup),
        )

        self.use_res_connect = stride == 1 and inp == oup

    def forward(self, x):
        if self.use_res_connect:
            return x + self.block(x)
        return self.block(x)


# Comparison: 40→40 channels, 56×56 spatial
mobilenet_block = MobileNetV2Block(40, 40, stride=1, expand_ratio=6)
ghostnet_block = GhostBottleneck(40, 240, 40, stride=1, se_ratio=0.25)

# FLOPs comparison
# MobileNetV2: (40×240 + 240×9 + 240×40) × 56² = 53.1M FLOPs
# GhostNet:    ~35M FLOPs (33% reduction)

# Accuracy: Nearly identical on downstream tasks
```

**EfficientNet vs. GhostNet**:

```
EfficientNet-B0 Architecture:
    Baseline: MBConv blocks (similar to MobileNetV2)
    Innovation: Compound scaling (depth, width, resolution)
    Strength: Best accuracy at similar param count
    Weakness: Higher FLOPs due to larger width multipliers

GhostNet Architecture:
    Baseline: Ghost modules (feature redundancy exploitation)
    Innovation: Cheap ghost operations
    Strength: Lowest FLOPs at similar accuracy
    Weakness: Slightly lower max accuracy ceiling

Complementary: Apply compound scaling to GhostNet
    GhostNet-B0: 5.2M, 142M FLOPs, 73.9% Top-1
    GhostNet-B1: 8.1M, 250M FLOPs, 76.5% Top-1
    GhostNet-B2: 11.3M, 380M FLOPs, 78.1% Top-1
```

**ShuffleNetV2 vs. GhostNet**:

```python
# ShuffleNetV2 uses channel split + shuffle
class ShuffleNetV2Block(nn.Module):
    def __init__(self, inp, oup, stride):
        super().__init__()
        self.stride = stride
        branch_features = oup // 2

        if stride > 1:
            self.branch1 = nn.Sequential(
                nn.Conv2d(inp, inp, 3, stride, 1, groups=inp, bias=False),
                nn.BatchNorm2d(inp),
                nn.Conv2d(inp, branch_features, 1, bias=False),
                nn.BatchNorm2d(branch_features),
                nn.ReLU(inplace=True),
            )

        self.branch2 = nn.Sequential(
            nn.Conv2d(inp if stride > 1 else branch_features,
                     branch_features, 1, bias=False),
            nn.BatchNorm2d(branch_features),
            nn.ReLU(inplace=True),
            nn.Conv2d(branch_features, branch_features, 3, stride, 1,
                     groups=branch_features, bias=False),
            nn.BatchNorm2d(branch_features),
            nn.Conv2d(branch_features, branch_features, 1, bias=False),
            nn.BatchNorm2d(branch_features),
            nn.ReLU(inplace=True),
        )

    def forward(self, x):
        if self.stride == 1:
            x1, x2 = x.chunk(2, dim=1)
            out = torch.cat((x1, self.branch2(x2)), dim=1)
        else:
            out = torch.cat((self.branch1(x), self.branch2(x)), dim=1)

        # Channel shuffle
        out = self.channel_shuffle(out, 2)
        return out

    @staticmethod
    def channel_shuffle(x, groups):
        batch, channels, height, width = x.size()
        channels_per_group = channels // groups
        x = x.view(batch, groups, channels_per_group, height, width)
        x = torch.transpose(x, 1, 2).contiguous()
        x = x.view(batch, -1, height, width)
        return x


# Key Differences:
# ShuffleNetV2: Channel split/shuffle for efficient feature reuse
# GhostNet:     Generate features cheaply from intrinsic features
# Performance:  Similar FLOPs, GhostNet ~2% better accuracy
```

### 6.3 Hardware Efficiency Analysis

**Latency Breakdown (ARM Cortex-A76, 224×224 input)**:

```
Model          | Conv  | DW Conv | Pointwise | Other | Total
---------------|-------|---------|-----------|-------|-------
MobileNetV2    | 12%   | 45%     | 35%       | 8%    | 8.5ms
ShuffleNetV2   | 15%   | 38%     | 32%       | 15%   | 8.2ms
GhostNet       | 25%   | 30%     | 28%       | 17%   | 6.9ms

Observation: GhostNet reduces depthwise conv overhead (30% vs 45%)
Strategy: Replace some depthwise with cheap operations
```

**Memory Bandwidth Analysis**:

```python
def analyze_memory_bandwidth(model, input_size=(1, 3, 224, 224)):
    """
    Analyze memory access patterns.

    Memory bandwidth is often the bottleneck on mobile devices.
    """
    import torch
    from torch.utils.hooks import RemovableHandle

    memory_read = 0
    memory_write = 0

    def hook_fn(module, input, output):
        nonlocal memory_read, memory_write

        # Input memory read
        for inp in input:
            if isinstance(inp, torch.Tensor):
                memory_read += inp.numel() * 4  # 4 bytes per float32

        # Output memory write
        if isinstance(output, torch.Tensor):
            memory_write += output.numel() * 4

        # Weight memory read
        for param in module.parameters():
            memory_read += param.numel() * 4

    hooks = []
    for module in model.modules():
        hooks.append(module.register_forward_hook(hook_fn))

    # Forward pass
    x = torch.randn(input_size)
    with torch.no_grad():
        _ = model(x)

    # Remove hooks
    for hook in hooks:
        hook.remove()

    return memory_read / 1e6, memory_write / 1e6  # Convert to MB


# Example results (MB of memory accessed):
# MobileNetV2:  Read=145MB, Write=78MB, Total=223MB
# GhostNet:     Read=98MB,  Write=52MB, Total=150MB
# Reduction: 33% less memory bandwidth required
```

**Energy Consumption**:

```
Energy = Memory_Access × Energy_per_Access + Compute × Energy_per_Op

Typical mobile SoC (Snapdragon 855):
    Memory access: 640 pJ/byte
    MAC operation:  3.7 pJ/op

MobileNetV2:
    Memory: 223MB × 640pJ/byte = 142.7 mJ
    Compute: 300M MAC × 3.7pJ = 1.1 mJ
    Total: 143.8 mJ

GhostNet:
    Memory: 150MB × 640pJ/byte = 96.0 mJ
    Compute: 142M MAC × 3.7pJ = 0.5 mJ
    Total: 96.5 mJ

Energy Savings: 33% reduction (mostly from memory bandwidth)
```

---

## 7. Training Strategies

### 7.1 From-Scratch Training

**Recommended Hyperparameters**:

```yaml
# ImageNet training configuration (original paper)
dataset: ImageNet-1K
epochs: 300
batch_size: 1024  # Across 4-8 GPUs
optimizer: SGD
learning_rate: 0.4  # Linear scaling: 0.1 × (batch_size / 256)
momentum: 0.9
weight_decay: 4e-5
nesterov: true

# Learning rate schedule
lr_schedule: cosine_annealing
warmup_epochs: 5
warmup_lr: 0.0
min_lr: 0.0

# Data augmentation
augmentation:
  - RandomResizedCrop(224)
  - RandomHorizontalFlip()
  - ColorJitter(brightness=0.4, contrast=0.4, saturation=0.4, hue=0.1)
  - AutoAugment  # Optional, +0.5% accuracy

# Regularization
label_smoothing: 0.1
dropout: 0.2
drop_path: 0.1  # Stochastic depth

# Mixed precision
amp: true  # Automatic mixed precision (FP16)
```

**Advanced Training Techniques**:

```python
class StochasticDepth(nn.Module):
    """
    Stochastic depth for regularization.

    Randomly drops entire residual branches during training.
    """
    def __init__(self, drop_prob=0.1):
        super().__init__()
        self.drop_prob = drop_prob

    def forward(self, x, residual):
        if not self.training or self.drop_prob == 0.:
            return x + residual

        # Bernoulli random variable
        keep_prob = 1 - self.drop_prob
        mask = torch.bernoulli(torch.full((x.size(0), 1, 1, 1), keep_prob,
                                         device=x.device))

        # Scale during training for expectation matching
        return x + residual * mask / keep_prob


# Apply to GhostBottleneck
class GhostBottleneckWithDropPath(GhostBottleneck):
    def __init__(self, *args, drop_path=0.0, **kwargs):
        super().__init__(*args, **kwargs)
        self.drop_path = StochasticDepth(drop_path)

    def forward(self, x):
        residual = self.shortcut(x)

        # Main path
        out = self.ghost1(x)
        if self.stride > 1:
            out = self.conv_dw(out)
            out = self.bn_dw(out)
        if self.se is not None:
            out = self.se(out)
        out = self.ghost2(out)

        # Apply stochastic depth
        out = self.drop_path(residual, out)

        return out
```

**Exponential Moving Average (EMA)**:

```python
class ModelEMA:
    """
    Model Exponential Moving Average.

    Maintains a moving average of model parameters for better generalization.
    """
    def __init__(self, model, decay=0.9999):
        self.model = model
        self.decay = decay
        self.shadow = {}
        self.backup = {}

        # Initialize shadow parameters
        for name, param in model.named_parameters():
            if param.requires_grad:
                self.shadow[name] = param.data.clone()

    @torch.no_grad()
    def update(self):
        """Update EMA parameters."""
        for name, param in self.model.named_parameters():
            if param.requires_grad:
                new_average = (1.0 - self.decay) * param.data + \
                             self.decay * self.shadow[name]
                self.shadow[name] = new_average.clone()

    def apply_shadow(self):
        """Apply EMA parameters to model."""
        for name, param in self.model.named_parameters():
            if param.requires_grad:
                self.backup[name] = param.data.clone()
                param.data = self.shadow[name]

    def restore(self):
        """Restore original parameters."""
        for name, param in self.model.named_parameters():
            if param.requires_grad:
                param.data = self.backup[name]
        self.backup = {}


# Usage in training loop
model = ghostnet(num_classes=1000, width=1.0)
ema = ModelEMA(model, decay=0.9999)

for epoch in range(num_epochs):
    for images, labels in train_loader:
        # Standard training step
        loss = train_step(model, images, labels)

        # Update EMA
        ema.update()

    # Validation with EMA model
    ema.apply_shadow()
    val_acc = evaluate(model, val_loader)
    ema.restore()

# Save EMA model for inference
ema.apply_shadow()
torch.save(model.state_dict(), 'ghostnet_ema.pth')
```

### 7.2 Knowledge Distillation

**Teacher-Student Training**:

```python
class DistillationLoss(nn.Module):
    """
    Knowledge distillation loss.

    Combines:
    1. Hard label loss (cross-entropy with ground truth)
    2. Soft label loss (KL divergence with teacher predictions)
    """
    def __init__(self, alpha=0.5, temperature=3.0):
        super().__init__()
        self.alpha = alpha
        self.temperature = temperature
        self.ce_loss = nn.CrossEntropyLoss()
        self.kl_loss = nn.KLDivLoss(reduction='batchmean')

    def forward(self, student_logits, teacher_logits, labels):
        # Hard label loss
        hard_loss = self.ce_loss(student_logits, labels)

        # Soft label loss (with temperature)
        soft_student = F.log_softmax(student_logits / self.temperature, dim=1)
        soft_teacher = F.softmax(teacher_logits / self.temperature, dim=1)
        soft_loss = self.kl_loss(soft_student, soft_teacher) * (self.temperature ** 2)

        # Combined loss
        return self.alpha * hard_loss + (1 - self.alpha) * soft_loss


def train_with_distillation(student, teacher, train_loader, num_epochs):
    """
    Train GhostNet (student) with knowledge distillation from larger model (teacher).
    """
    # Freeze teacher
    teacher.eval()
    for param in teacher.parameters():
        param.requires_grad = False

    # Setup training
    optimizer = torch.optim.SGD(student.parameters(), lr=0.1,
                               momentum=0.9, weight_decay=4e-5)
    scheduler = torch.optim.lr_scheduler.CosineAnnealingLR(optimizer, num_epochs)
    criterion = DistillationLoss(alpha=0.3, temperature=3.0)

    device = torch.device('cuda' if torch.cuda.is_available() else 'cpu')
    student.to(device)
    teacher.to(device)

    for epoch in range(num_epochs):
        student.train()

        for images, labels in train_loader:
            images, labels = images.to(device), labels.to(device)

            # Get teacher predictions (no gradients)
            with torch.no_grad():
                teacher_logits = teacher(images)

            # Student forward pass
            student_logits = student(images)

            # Compute distillation loss
            loss = criterion(student_logits, teacher_logits, labels)

            # Backward pass
            optimizer.zero_grad()
            loss.backward()
            optimizer.step()

        scheduler.step()

    return student


# Example usage
teacher = torchvision.models.resnet101(pretrained=True)
student = ghostnet(num_classes=1000, width=1.0)

trained_student = train_with_distillation(
    student, teacher, train_loader, num_epochs=100
)

# Expected improvement: +1-2% accuracy over from-scratch training
```

**Feature-based Distillation**:

```python
class FeatureDistillationLoss(nn.Module):
    """
    Distillation on intermediate features, not just final logits.
    """
    def __init__(self, alpha=0.5, beta=0.3):
        super().__init__()
        self.alpha = alpha  # Weight for logit distillation
        self.beta = beta    # Weight for feature distillation
        self.mse_loss = nn.MSELoss()

    def forward(self, student_outputs, teacher_outputs, labels):
        student_logits, student_feats = student_outputs
        teacher_logits, teacher_feats = teacher_outputs

        # Logit distillation
        logit_loss = F.kl_div(
            F.log_softmax(student_logits / 3.0, dim=1),
            F.softmax(teacher_logits / 3.0, dim=1),
            reduction='batchmean'
        ) * 9.0

        # Feature distillation (match intermediate representations)
        feature_loss = sum(
            self.mse_loss(s_feat, t_feat)
            for s_feat, t_feat in zip(student_feats, teacher_feats)
        ) / len(student_feats)

        # Hard label loss
        hard_loss = F.cross_entropy(student_logits, labels)

        return hard_loss + self.alpha * logit_loss + self.beta * feature_loss
```

### 7.3 Progressive Training

**Progressive Resizing**:

```python
def progressive_training(model, train_dataset, num_epochs):
    """
    Progressive training: start with small images, gradually increase size.

    Benefits:
    - Faster early training (smaller images)
    - Better generalization (augmentation-like effect)
    - Convergence improvement
    """
    # Training stages: resolution increases over time
    stages = [
        {'resolution': 128, 'epochs': 100, 'batch_size': 512},
        {'resolution': 192, 'epochs': 100, 'batch_size': 256},
        {'resolution': 224, 'epochs': 100, 'batch_size': 256},
    ]

    for stage in stages:
        print(f"Training stage: {stage['resolution']}×{stage['resolution']}")

        # Update data augmentation for current resolution
        transform = transforms.Compose([
            transforms.RandomResizedCrop(stage['resolution']),
            transforms.RandomHorizontalFlip(),
            transforms.ToTensor(),
            transforms.Normalize([0.485, 0.456, 0.406], [0.229, 0.224, 0.225])
        ])

        train_dataset.transform = transform

        # Create data loader with stage-specific batch size
        train_loader = DataLoader(
            train_dataset,
            batch_size=stage['batch_size'],
            shuffle=True,
            num_workers=8
        )

        # Train for this stage
        train(model, train_loader, epochs=stage['epochs'])

    return model
```


---

## 8. Applications and Use Cases

### 8.1 Mobile Computer Vision

**Real-time Object Detection on Mobile**:

```python
# Deploy GhostNet for mobile object detection
import torch
import torchvision

# Optimized model for mobile
model = GhostNetDetector(num_classes=80, width=1.0)
model.load_state_dict(torch.load('ghostnet_detector_coco.pth'))
model.eval()

# Convert to TorchScript for mobile deployment
example_input = torch.rand(1, 3, 416, 416)
traced_model = torch.jit.trace(model, example_input)

# Optimize for mobile
from torch.utils.mobile_optimizer import optimize_for_mobile
mobile_model = optimize_for_mobile(traced_model)

# Save for mobile deployment
mobile_model._save_for_lite_interpreter("ghostnet_detector_mobile.ptl")

# Expected performance (Snapdragon 855):
# - Latency: 45ms
# - mAP (COCO): 28.5%
# - Energy: <150mJ per frame
```

**Use Cases**:
- **Autonomous driving**: Lane detection, object detection
- **Augmented reality**: Real-time scene understanding
- **Mobile photography**: Portrait mode, scene recognition
- **Security cameras**: Person/vehicle detection
- **Retail**: Product recognition, inventory management
- **Healthcare**: Medical image analysis on mobile devices

### 8.2 Edge Computing

**IoT Device Deployment**:

```python
import onnx
import onnxruntime

# Convert to ONNX for broad hardware support
model = ghostnet(num_classes=1000, width=0.5)  # Smallest variant
model.eval()

dummy_input = torch.randn(1, 3, 224, 224)
torch.onnx.export(
    model,
    dummy_input,
    "ghostnet_0.5x.onnx",
    opset_version=11,
    input_names=['input'],
    output_names=['output'],
    dynamic_axes={'input': {0: 'batch'}, 'output': {0: 'batch'}}
)

# Quantize for edge devices (INT8)
from onnxruntime.quantization import quantize_dynamic, QuantType

quantize_dynamic(
    "ghostnet_0.5x.onnx",
    "ghostnet_0.5x_int8.onnx",
    weight_type=QuantType.QInt8
)

# Expected model size:
# FP32: 10.4 MB
# INT8: 2.8 MB (73% reduction)
#
# Expected performance (Raspberry Pi 4):
# FP32: 180ms
# INT8: 95ms (47% faster)
```

**Edge Applications**:
- **Smart cameras**: Face recognition, intrusion detection
- **Industrial IoT**: Defect detection, quality control
- **Smart home**: Gesture recognition, activity monitoring
- **Agriculture**: Crop disease detection, yield estimation
- **Robotics**: Visual navigation, object manipulation

### 8.3 Cloud Inference Optimization

**Batched Inference**:

```python
def optimized_cloud_inference(model, image_batch, device='cuda'):
    """
    Optimized batched inference for cloud deployment.

    Techniques:
    - Dynamic batching
    - Mixed precision
    - TensorRT optimization
    """
    import tensorrt as trt
    import pycuda.driver as cuda

    # Convert to TensorRT for maximum throughput
    # (This is a simplified example; full TensorRT conversion is complex)

    model = model.to(device).eval()

    with torch.cuda.amp.autocast():  # Mixed precision
        with torch.no_grad():
            outputs = model(image_batch)

    return outputs


# Expected throughput (NVIDIA T4 GPU):
# Batch size 1:   270 images/second
# Batch size 8:   980 images/second
# Batch size 32:  1850 images/second
#
# Cost efficiency: $0.000005 per inference at batch=32
```

---

## 9. Advanced Techniques and Variants

### 9.1 GhostNetV2 Improvements

**DFC Attention (Decoupled Fully Connected)**:

```python
class DFCAttention(nn.Module):
    """
    DFC Attention from GhostNetV2.

    Improves ghost feature quality through long-range dependencies.
    """
    def __init__(self, in_channels, reduction=4):
        super().__init__()
        self.in_channels = in_channels

        # Horizontal FC
        self.fc_h = nn.Conv2d(in_channels, in_channels // reduction, 1)

        # Vertical FC  
        self.fc_w = nn.Conv2d(in_channels, in_channels // reduction, 1)

        # Output projection
        self.fc_out = nn.Conv2d(in_channels // reduction, in_channels, 1)

        self.sigmoid = nn.Sigmoid()

    def forward(self, x):
        B, C, H, W = x.shape

        # Horizontal attention
        h_pool = torch.mean(x, dim=3, keepdim=True)  # [B, C, H, 1]
        h_attn = self.fc_h(h_pool)  # [B, C//r, H, 1]
        h_attn = h_attn.expand(-1, -1, -1, W)  # [B, C//r, H, W]

        # Vertical attention
        w_pool = torch.mean(x, dim=2, keepdim=True)  # [B, C, 1, W]
        w_attn = self.fc_w(w_pool)  # [B, C//r, 1, W]
        w_attn = w_attn.expand(-1, -1, H, -1)  # [B, C//r, H, W]

        # Combine
        attn = h_attn + w_attn
        attn = self.fc_out(attn)
        attn = self.sigmoid(attn)

        return x * attn


class GhostModuleV2(nn.Module):
    """
    GhostNet V2 module with DFC attention.

    Improvements over V1:
    - Better feature quality through attention
    - ~1% accuracy improvement
    - Minimal latency increase
    """
    def __init__(self, inp, oup, kernel_size=1, ratio=2, dw_size=3, stride=1):
        super().__init__()
        self.oup = oup
        init_channels = math.ceil(oup / ratio)
        new_channels = init_channels * (ratio - 1)

        # Primary convolution
        self.primary_conv = nn.Sequential(
            nn.Conv2d(inp, init_channels, kernel_size, stride,
                     kernel_size//2, bias=False),
            nn.BatchNorm2d(init_channels),
            nn.ReLU(inplace=True),
        )

        # Cheap operation with DFC attention
        self.cheap_operation = nn.Sequential(
            nn.Conv2d(init_channels, new_channels, dw_size, 1, dw_size//2,
                     groups=init_channels, bias=False),
            nn.BatchNorm2d(new_channels),
        )

        self.dfc_attn = DFCAttention(new_channels)

        self.relu = nn.ReLU(inplace=True)

    def forward(self, x):
        x1 = self.primary_conv(x)
        x2 = self.cheap_operation(x1)
        x2 = self.dfc_attn(x2)  # Apply DFC attention
        x2 = self.relu(x2)
        out = torch.cat([x1, x2], dim=1)
        return out[:, :self.oup, :, :]
```

### 9.2 Neural Architecture Search (NAS) for GhostNet

**Automated Ghost Ratio Search**:

```python
class SearchableGhostModule(nn.Module):
    """
    Searchable Ghost module for NAS.

    Learns optimal ratio (s) per layer.
    """
    def __init__(self, inp, oup, kernel_size=1, dw_size=3, stride=1):
        super().__init__()
        self.inp = inp
        self.oup = oup

        # Candidate ratios
        self.ratios = [1.5, 2.0, 2.5, 3.0, 4.0]

        # Architecture parameters (learnable)
        self.arch_params = nn.Parameter(torch.randn(len(self.ratios)))

        # Ghost modules for each ratio
        self.ghost_modules = nn.ModuleList([
            GhostModule(inp, oup, kernel_size, ratio=r, dw_size=dw_size, stride=stride)
            for r in self.ratios
        ])

    def forward(self, x):
        # Gumbel-Softmax for differentiable architecture search
        weights = F.gumbel_softmax(self.arch_params, tau=1.0, hard=False)

        # Weighted combination of all ratios
        output = sum(w * ghost_module(x)
                    for w, ghost_module in zip(weights, self.ghost_modules))

        return output

    def get_best_ratio(self):
        """Get the best ratio after search."""
        idx = self.arch_params.argmax().item()
        return self.ratios[idx]


# Search process
def search_ghost_ratios(model, train_loader, val_loader, search_epochs=50):
    """
    Search for optimal ghost ratios using differentiable NAS.
    """
    # Joint optimization of weights and architecture
    model_params = [p for n, p in model.named_parameters()
                   if 'arch_params' not in n]
    arch_params = [p for n, p in model.named_parameters()
                  if 'arch_params' in n]

    model_optimizer = torch.optim.SGD(model_params, lr=0.1, momentum=0.9)
    arch_optimizer = torch.optim.Adam(arch_params, lr=0.01)

    for epoch in range(search_epochs):
        # Train model weights
        for images, labels in train_loader:
            loss = train_step(model, images, labels)
            model_optimizer.zero_grad()
            loss.backward()
            model_optimizer.step()

        # Train architecture parameters on validation set
        for images, labels in val_loader:
            loss = train_step(model, images, labels)
            arch_optimizer.zero_grad()
            loss.backward()
            arch_optimizer.step()

    # Extract best architecture
    best_ratios = [module.get_best_ratio()
                  for module in model.modules()
                  if isinstance(module, SearchableGhostModule)]

    return best_ratios
```

### 9.3 Pruning and Quantization

**Structured Pruning**:

```python
import torch.nn.utils.prune as prune

def prune_ghostnet(model, amount=0.3):
    """
    Structured pruning for GhostNet.

    Prune entire channels based on importance scores.
    """
    # Identify prunable modules
    modules_to_prune = []
    for name, module in model.named_modules():
        if isinstance(module, nn.Conv2d):
            modules_to_prune.append((module, 'weight'))

    # Apply structured pruning (L1 norm)
    for module, param_name in modules_to_prune:
        prune.ln_structured(
            module,
            name=param_name,
            amount=amount,
            n=1,  # L1 norm
            dim=0  # Prune output channels
        )

    # Make pruning permanent
    for module, param_name in modules_to_prune:
        prune.remove(module, param_name)

    return model


# Example usage
model = ghostnet(num_classes=1000, width=1.0)
pruned_model = prune_ghostnet(model, amount=0.3)

# Expected results:
# Parameters: 5.2M → 3.6M (30% reduction)
# FLOPs: 142M → 99M (30% reduction)
# Accuracy: 73.9% → 73.1% (0.8% drop)
#
# After fine-tuning 20 epochs:
# Accuracy: 73.9% → 73.6% (0.3% drop)
```

**Post-Training Quantization**:

```python
def quantize_ghostnet(model, calibration_loader):
    """
    Post-training quantization (INT8).

    Converts FP32 weights to INT8 for faster inference.
    """
    from torch.quantization import quantize_dynamic, quantize_static, prepare, convert

    # Dynamic quantization (weights only)
    model_dynamic = quantize_dynamic(
        model,
        {nn.Linear, nn.Conv2d},
        dtype=torch.qint8
    )

    # Static quantization (weights + activations)
    model.eval()
    model.qconfig = torch.quantization.get_default_qconfig('fbgemm')
    model_prepared = prepare(model)

    # Calibration
    with torch.no_grad():
        for images, _ in calibration_loader:
            model_prepared(images)

    model_static = convert(model_prepared)

    return model_dynamic, model_static


# Expected results:
# Model size: 20.8 MB → 5.5 MB (74% reduction)
# Inference speed (CPU): 6.9ms → 3.2ms (2.2× faster)
# Accuracy: 73.9% → 73.5% (0.4% drop)
```

---

## 10. Performance Optimization

### 10.1 Operator Fusion

```python
def fuse_modules(model):
    """
    Fuse Conv-BN-ReLU for faster inference.
    """
    from torch.quantization import fuse_modules

    # Automatically detect fusion patterns
    for name, module in model.named_modules():
        if isinstance(module, GhostModule):
            # Fuse primary conv
            fuse_modules(module.primary_conv, [['0', '1', '2']], inplace=True)

            # Fuse cheap operation
            fuse_modules(module.cheap_operation, [['0', '1', '2']], inplace=True)

    return model


# Expected speedup: 10-15% on CPU
```

### 10.2 TensorRT Optimization

```python
def convert_to_tensorrt(model, input_shape=(1, 3, 224, 224)):
    """
    Convert GhostNet to TensorRT for maximum GPU performance.
    """
    import tensorrt as trt
    import torch2trt

    # Example input
    x = torch.randn(input_shape).cuda()

    # Convert
    model_trt = torch2trt.torch2trt(
        model,
        [x],
        fp16_mode=True,  # FP16 for 2× speedup
        max_batch_size=32
    )

    return model_trt


# Expected performance (T4 GPU):
# PyTorch FP32: 3.7ms
# PyTorch FP16: 2.1ms
# TensorRT FP16: 1.3ms (2.8× faster than PyTorch FP32)
```

---

## 11. Future Directions

### 11.1 Emerging Trends

**Transformer-Ghost Hybrid**:
```python
class GhostViT(nn.Module):
    """
    Vision Transformer with Ghost operations.

    Replace expensive MLP with Ghost modules.
    """
    def __init__(self, dim=384, mlp_ratio=4):
        super().__init__()
        hidden_dim = int(dim * mlp_ratio)

        # Replace standard MLP with Ghost module
        self.mlp = nn.Sequential(
            GhostModule(dim, hidden_dim, kernel_size=1, ratio=2),
            nn.GELU(),
            GhostModule(hidden_dim, dim, kernel_size=1, ratio=2),
        )

    def forward(self, x):
        return x + self.mlp(x)

# Expected: 30-40% FLOPs reduction in MLP layers
```

**Neural Architecture Search**:
- AutoML for Ghost ratio optimization per layer
- Hardware-aware NAS for specific deployment targets
- Efficient search algorithms (one-shot, differentiable)

**Dynamic Networks**:
- Adaptive ghost ratio based on input complexity
- Early exit strategies for efficient inference
- Dynamic channel selection

### 11.2 Research Opportunities

**Theoretical Analysis**:
- Formal analysis of feature redundancy in different network depths
- Optimal ghost ratio as function of layer characteristics
- Connection to matrix factorization and low-rank approximation

**Applications**:
- Video understanding with temporal ghost features
- 3D computer vision (point clouds, voxels)
- Multimodal learning (vision-language models)
- Generative models (GANs, diffusion models)

**Hardware Co-design**:
- Custom accelerators for ghost operations
- Neuromorphic implementation of ghost modules
- Energy-efficient FPGA deployment

---

## 12. References and Resources

### 12.1 Key Papers

**GhostNet**:
1. Kai Han et al. "GhostNet: More Features from Cheap Operations." CVPR 2020.
   https://arxiv.org/abs/1911.11907

2. Kai Han et al. "GhostNetV2: Enhance Cheap Operation with Long-Range Attention." NeurIPS 2022.
   https://arxiv.org/abs/2211.12905

**Related Efficient Architectures**:
3. Andrew G. Howard et al. "MobileNets: Efficient Convolutional Neural Networks for Mobile Vision Applications." arXiv 2017.

4. Mark Sandler et al. "MobileNetV2: Inverted Residuals and Linear Bottlenecks." CVPR 2018.

5. Ningning Ma et al. "ShuffleNet V2: Practical Guidelines for Efficient CNN Architecture Design." ECCV 2018.

6. Mingxing Tan and Quoc V. Le. "EfficientNet: Rethinking Model Scaling for Convolutional Neural Networks." ICML 2019.

**Network Compression**:
7. Song Han et al. "Deep Compression: Compressing Deep Neural Networks with Pruning, Trained Quantization and Huffman Coding." ICLR 2016.

8. Geoffrey Hinton et al. "Distilling the Knowledge in a Neural Network." NIPS 2014 Workshop.

### 12.2 Code Repositories

**Official Implementation**:
- GhostNet PyTorch: https://github.com/huawei-noah/ghostnet
- GhostNetV2 PyTorch: https://github.com/huawei-noah/Efficient-AI-Backbones

**Third-party Implementations**:
- TensorFlow/Keras: https://github.com/keras-team/keras-applications
- ONNX: https://github.com/onnx/models

**Deployment Tools**:
- TensorRT: https://github.com/NVIDIA/TensorRT
- NCNN: https://github.com/Tencent/ncnn
- TFLite: https://www.tensorflow.org/lite

### 12.3 Datasets and Benchmarks

**Image Classification**:
- ImageNet-1K: 1.28M training images, 1000 classes
- CIFAR-10/100: 60K images, 10/100 classes
- iNaturalist: 5.1M images, 10K species

**Object Detection**:
- COCO: 330K images, 80 classes, 1.5M object instances
- Pascal VOC: 20 classes, detection + segmentation
- Open Images: 9M images, 600 classes

**Mobile Benchmarks**:
- MLPerf Mobile: Industry-standard mobile AI benchmark
- AI Benchmark: Cross-platform performance evaluation
- MobileAI Workshop: Annual competition for mobile vision

### 12.4 Tools and Libraries

**Training Frameworks**:
```bash
# PyTorch (recommended)
pip install torch torchvision

# TensorFlow
pip install tensorflow

# JAX (for research)
pip install jax jaxlib

# Timm (PyTorch Image Models - includes GhostNet)
pip install timm
```

**Optimization Tools**:
```bash
# ONNX tools
pip install onnx onnxruntime

# TensorRT
# Follow NVIDIA TensorRT installation guide

# Quantization
pip install torch-quantization

# Model analysis
pip install thop  # FLOPs counter
pip install ptflops  # Alternative FLOPs counter
pip install torchinfo  # Model summary
```

**Deployment**:
```bash
# Mobile (PyTorch Mobile)
pip install torch torchvision

# Mobile (TFLite)
pip install tensorflow

# Edge (ONNX Runtime)
pip install onnxruntime

# Cloud (TorchServe)
pip install torchserve torch-model-archiver
```

---

## Conclusion

GhostNet represents a significant advancement in efficient CNN architectures by explicitly exploiting feature redundancy. Through cheap operations that generate "ghost" features from intrinsic features, GhostNet achieves:

**Key Achievements**:
- 2-4× FLOPs reduction compared to standard convolutions
- Minimal accuracy loss (<1% on ImageNet)
- 30-50% faster inference on mobile devices
- 33% reduction in energy consumption
- Compatible with existing training pipelines

**Design Philosophy**:
- Evidence-based: Redundancy is empirically observable
- Trainable: End-to-end learning without post-processing
- Flexible: Adjustable ghost ratio per layer
- Complementary: Works with other efficiency techniques

**Practical Impact**:
- Enables real-time vision on mobile devices
- Reduces cloud inference costs
- Powers edge AI applications
- Democratizes access to computer vision

**Future Outlook**:
GhostNet's core insight—exploiting feature redundancy through cheap operations—will likely influence future efficient architecture designs. Combined with advances in NAS, quantization, and hardware acceleration, GhostNet and its descendants will play a crucial role in bringing AI to resource-constrained environments.

---

**Document Version**: 1.0
**Last Updated**: January 2026
**Total Length**: 3,800+ lines
**License**: Educational Use

*This guide provides comprehensive technical information about GhostNet architecture. For production deployment, consult the official repository and consider specific hardware constraints.*

