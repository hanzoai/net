import math
from collections import OrderedDict
from typing import List, Optional, Tuple, Dict
from net.helpers import exo_text, pretty_print_bytes, pretty_print_bytes_per_second
from net.topology.topology import Topology
from net.topology.partitioning_strategy import Partition
from net.download.download_progress import RepoProgressEvent
from net.topology.device_capabilities import UNKNOWN_DEVICE_CAPABILITIES
from rich.console import Console, Group
from rich.text import Text
from rich.live import Live
from rich.style import Style
from rich.table import Table
from rich.layout import Layout
from rich.syntax import Syntax
from rich.panel import Panel
from rich.markdown import Markdown


class TopologyViz:
  def __init__(self, chatgpt_api_endpoints: List[str] = [], web_chat_urls: List[str] = []):
    self.chatgpt_api_endpoints = chatgpt_api_endpoints
    self.web_chat_urls = web_chat_urls
    self.topology = Topology()
    self.partitions: List[Partition] = []
    self.node_id = None
    self.node_download_progress: Dict[str, RepoProgressEvent] = {}
    self.requests: OrderedDict[str, Tuple[str, str]] = {}

    self.console = Console()
    self.layout = Layout()
    self.layout.split(Layout(name="main"), Layout(name="prompt_output", size=15), Layout(name="download", size=25))
    self.main_panel = Panel(self._generate_main_layout(), title="Hanzo Network (0 nodes)", border_style="bright_yellow")
    self.prompt_output_panel = Panel("", title="Prompt and Output", border_style="green")
    self.download_panel = Panel("", title="Download Progress", border_style="cyan")
    self.layout["main"].update(self.main_panel)
    self.layout["prompt_output"].update(self.prompt_output_panel)
    self.layout["download"].update(self.download_panel)

    # Initially hide the prompt_output panel
    self.layout["prompt_output"].visible = False
    self.live_panel = Live(self.layout, auto_refresh=False, console=self.console)
    self.live_panel.start()

  def update_visualization(self, topology: Topology, partitions: List[Partition], node_id: Optional[str] = None, node_download_progress: Dict[str, RepoProgressEvent] = {}):
    self.topology = topology
    self.partitions = partitions
    self.node_id = node_id
    if node_download_progress:
      self.node_download_progress = node_download_progress
    self.refresh()

  def update_prompt(self, request_id: str, prompt: Optional[str] = None):
    self.requests[request_id] = [prompt, self.requests.get(request_id, ["", ""])[1]]
    self.refresh()

  def update_prompt_output(self, request_id: str, output: Optional[str] = None):
    self.requests[request_id] = [self.requests.get(request_id, ["", ""])[0], output]
    self.refresh()

  def refresh(self):
    self.main_panel.renderable = self._generate_main_layout()
    # Update the panel title with the number of nodes and partitions
    node_count = len(self.topology.nodes)
    self.main_panel.title = f"Hanzo Network ({node_count} node{'s' if node_count != 1 else ''})"

    # Update and show/hide prompt and output panel
    if any(r[0] or r[1] for r in self.requests.values()):
      self.prompt_output_panel = self._generate_prompt_output_layout()
      self.layout["prompt_output"].update(self.prompt_output_panel)
      self.layout["prompt_output"].visible = True
    else:
      self.layout["prompt_output"].visible = False

    # Only show download_panel if there are in-progress downloads
    if any(progress.status == "in_progress" for progress in self.node_download_progress.values()):
      self.download_panel.renderable = self._generate_download_layout()
      self.layout["download"].visible = True
    else:
      self.layout["download"].visible = False

    self.live_panel.update(self.layout, refresh=True)

  def _generate_prompt_output_layout(self) -> Panel:
    content = []
    requests = list(self.requests.values())[-3:]  # Get the 3 most recent requests
    max_width = self.console.width - 6  # Full width minus padding and icon

    # Calculate available height for content
    panel_height = 15  # Fixed panel height
    available_lines = panel_height - 2  # Subtract 2 for panel borders
    lines_per_request = available_lines // len(requests) if requests else 0

    for (prompt, output) in reversed(requests):
      prompt_icon, output_icon = "💬️", "🤖"

      # Equal space allocation for prompt and output
      max_prompt_lines = lines_per_request // 2
      max_output_lines = lines_per_request - max_prompt_lines - 1  # -1 for spacing

      # Process prompt
      prompt_lines = []
      for line in prompt.split('\n'):
        words = line.split()
        current_line = []
        current_length = 0

        for word in words:
          if current_length + len(word) + 1 <= max_width:
            current_line.append(word)
            current_length += len(word) + 1
          else:
            if current_line:
              prompt_lines.append(' '.join(current_line))
            current_line = [word]
            current_length = len(word)

        if current_line:
          prompt_lines.append(' '.join(current_line))

      # Truncate prompt if needed
      if len(prompt_lines) > max_prompt_lines:
        prompt_lines = prompt_lines[:max_prompt_lines]
        if prompt_lines:
          last_line = prompt_lines[-1]
          if len(last_line) + 4 <= max_width:
            prompt_lines[-1] = last_line + " ..."
          else:
            prompt_lines[-1] = last_line[:max_width-4] + " ..."

      prompt_text = Text(f"{prompt_icon} ", style="bold bright_blue")
      prompt_text.append('\n'.join(prompt_lines), style="white")
      content.append(prompt_text)

      # Process output with similar word wrapping
      if output:  # Only process output if it exists
        output_lines = []
        for line in output.split('\n'):
          words = line.split()
          current_line = []
          current_length = 0

          for word in words:
            if current_length + len(word) + 1 <= max_width:
              current_line.append(word)
              current_length += len(word) + 1
            else:
              if current_line:
                output_lines.append(' '.join(current_line))
              current_line = [word]
              current_length = len(word)

          if current_line:
            output_lines.append(' '.join(current_line))

        # Truncate output if needed
        if len(output_lines) > max_output_lines:
          output_lines = output_lines[:max_output_lines]
          if output_lines:
            last_line = output_lines[-1]
            if len(last_line) + 4 <= max_width:
              output_lines[-1] = last_line + " ..."
            else:
              output_lines[-1] = last_line[:max_width-4] + " ..."

        output_text = Text(f"{output_icon} ", style="bold bright_magenta")
        output_text.append('\n'.join(output_lines), style="white")
        content.append(output_text)

      content.append(Text())  # Empty line between entries

    return Panel(
      Group(*content),
      title="",
      border_style="cyan",
      height=panel_height,
      expand=True
    )

  def _generate_main_layout(self) -> str:
    # Get terminal dimensions for responsive layout
    term_width = self.console.width - 4  # Account for panel borders
    term_height = min(48, self.console.height - 10)  # Adaptive height
    
    # Detect if we're in compact mode
    compact_mode = term_width < 80
    mobile_mode = term_width < 60
    
    # Adaptive sizing based on terminal width
    if mobile_mode:
      # Mobile mode for phones/very narrow terminals
      viz_width = max(40, term_width)
      radius_x = min(12, viz_width // 4)
      radius_y = 5
      term_height = min(35, term_height)  # Reasonable height for mobile
      show_qr = True  # Keep QR code even on mobile
      show_logo = False  # No large logo on mobile
      qr_position = 'below'  # Position QR below URLs
    elif compact_mode:
      # Compact mode for narrow terminals  
      viz_width = max(50, term_width)
      radius_x = min(18, viz_width // 4)
      radius_y = 7
      term_height = min(40, term_height)
      show_qr = True  # Keep QR in compact mode
      show_logo = True  # Keep logo but smaller
      qr_position = 'below'  # Position QR below URLs
    elif term_width < 120:
      # Medium mode
      viz_width = min(100, term_width)
      radius_x = min(25, viz_width // 3)
      radius_y = 10
      show_qr = True
      show_logo = True
      qr_position = 'right'  # QR on right side
    else:
      # Full mode for wide terminals
      viz_width = min(140, term_width)
      radius_x = min(35, viz_width // 4)
      radius_y = 12
      show_qr = True
      show_logo = True
      qr_position = 'right'  # QR on right side
    
    # Calculate center based on actual width
    center_x = viz_width // 2
    center_y = 24
    
    # Calculate visualization parameters
    num_partitions = len(self.partitions)

    # Generate visualization with adaptive width
    visualization = [[" " for _ in range(viz_width)] for _ in range(term_height)]

    # Add logo/title based on mode
    yellow_style = Style(color="bright_yellow")
    
    if mobile_mode:
      # Simple text title for mobile
      title = "HANZO"
      version = f"v{self._get_version_string()}"
      start_x = (viz_width - len(title)) // 2
      start_y = 1
      for j, char in enumerate(title):
        if 0 <= start_x + j < viz_width:
          visualization[start_y][start_x + j] = char
      # Add version below
      version_x = (viz_width - len(version)) // 2
      for j, char in enumerate(version):
        if 0 <= version_x + j < viz_width:
          visualization[start_y + 1][version_x + j] = char
    elif compact_mode:
      # Smaller ASCII art for compact mode
      compact_logo = [
        "HANZO",
        "━━━━━"
      ]
      for i, line in enumerate(compact_logo):
        if i + 1 < len(visualization):
          start_x = (viz_width - len(line)) // 2
          for j, char in enumerate(line):
            if 0 <= start_x + j < viz_width:
              visualization[i + 1][start_x + j] = char
    elif show_logo:
      # Full ASCII art logo for normal/wide mode
      exo_lines = exo_text.split("\n")
      max_line_length = max(len(line) for line in exo_lines) if exo_lines else 0
      for i, line in enumerate(exo_lines):
        if line.strip():  # Only process non-empty lines
          centered_line = line.center(max_line_length)
          start_x = (viz_width - max_line_length) // 2
          colored_line = Text(centered_line, style=yellow_style)
          for j, char in enumerate(str(colored_line)):
            if 0 <= start_x + j < viz_width and i < len(visualization):
              visualization[i][start_x + j] = char

    # Prepare QR code and URLs first to know dimensions
    qr_lines = []
    qr_width = 0
    web_url = ""
    api_url = ""
    
    if len(self.web_chat_urls) > 0:
      # Import helper to get best network IP
      from net.helpers import get_best_network_ip
      
      # Get the best network IP for mobile access
      best_ip = get_best_network_ip()
      
      # Extract port from first URL
      port = 52415  # default
      if self.web_chat_urls and len(self.web_chat_urls) > 0 and self.web_chat_urls[0]:
        try:
          from urllib.parse import urlparse
          parsed = urlparse(self.web_chat_urls[0])
          if parsed.port:
            port = parsed.port
        except:
          pass
      
      # Create URLs with best network IP
      web_url = f"http://{best_ip}:{port}"
      if len(self.chatgpt_api_endpoints) > 0:
        api_url = f"http://{best_ip}:{port}/v1/chat/completions"
      
      # Generate QR code if enabled
      if show_qr:
        try:
          import qrcode
          import io
          
          # Create QR code with ASCII art output - smaller border for smaller QR
          qr = qrcode.QRCode(border=1, box_size=1)
          qr.add_data(web_url)
          qr.make()
          
          # Use the built-in ASCII art generation with proper block characters
          f = io.StringIO()
          qr.print_ascii(out=f, tty=False, invert=False)
          qr_ascii = f.getvalue()
          
          # Split into lines and clean up
          qr_lines = qr_ascii.strip().split('\n')
          
          # Fix the first line alignment issue
          if qr_lines and not qr_lines[0].startswith(' '):
            qr_lines[0] = ' ' + qr_lines[0]
          
          # Clean up trailing whitespace
          qr_lines = [line.rstrip() for line in qr_lines]
          
          # Calculate QR code width
          qr_width = max(len(line) for line in qr_lines) if qr_lines else 0
          
        except ImportError:
          qr_lines = []
          qr_width = 0
        except Exception as e:
          qr_lines = []
          qr_width = 0
      else:
        # No QR code in compact mode
        qr_lines = []
        qr_width = 0
    
    # Add version information below the logo
    if mobile_mode:
      info_start_y = 3
    elif compact_mode:
      info_start_y = 4
    else:
      info_start_y = len(exo_text.split('\n')) + 1
    version_line = ""
    
    # Skip version display in mobile mode (already shown with title)
    if not mobile_mode:
      try:
        # Get hanzo-net version
        import importlib.metadata
        try:
          hanzo_net_version = importlib.metadata.version('hanzo-net')
        except:
          hanzo_net_version = "dev"
        
        # Try to get hanzo CLI version if available
        try:
          hanzo_version = importlib.metadata.version('hanzo')
          version_line = f"hanzo v{hanzo_version} | hanzo-net v{hanzo_net_version}"
        except:
          version_line = f"hanzo-net v{hanzo_net_version}"
        
        # Center the version line
        start_x = (viz_width - len(version_line)) // 2
        for j, char in enumerate(version_line):
          if 0 <= start_x + j < viz_width:
            visualization[info_start_y][start_x + j] = char
        info_start_y += 2
      except:
        pass
    
    # Responsive URL and QR display
    if mobile_mode:
      # Mobile: Compact URL display at top, QR below
      url_y = info_start_y
      if web_url:
        # Extract just IP and port for mobile
        from urllib.parse import urlparse
        parsed = urlparse(web_url)
        compact_url = f"{parsed.netloc}"
        # Center the URL
        url_x = (viz_width - len(compact_url)) // 2
        for j, char in enumerate(compact_url):
          if 0 <= url_x + j < viz_width and url_y < term_height:
            visualization[url_y][url_x + j] = char
        url_y += 2
      
      # Add QR code below URL in mobile mode
      if qr_lines and show_qr:
        # Center the QR code
        qr_start_x = max(0, (viz_width - qr_width) // 2)
        qr_start_y = url_y + 1
        
        for i, line in enumerate(qr_lines):
          if qr_start_y + i < term_height - 8:  # Leave room for GPU bar
            for j, char in enumerate(line):
              if 0 <= qr_start_x + j < viz_width:
                visualization[qr_start_y + i][qr_start_x + j] = char
        
        url_y = qr_start_y + len(qr_lines) + 1
    elif compact_mode:
      # Compact: URLs at top, QR below
      url_start_x = 2  # Smaller margin
      url_y = info_start_y
      
      if web_url:
        # Shorten URL for compact mode
        compact_label = "Web:"
        for j, char in enumerate(compact_label):
          if url_start_x + j < viz_width:
            visualization[url_y][url_start_x + j] = char
        
        # Put URL on same line if space permits
        url_text = web_url
        if len(compact_label) + len(url_text) + 1 > viz_width - 4:
          # URL on next line if too long
          url_y += 1
          url_x = url_start_x
        else:
          url_x = url_start_x + len(compact_label) + 1
        
        for j, char in enumerate(url_text):
          if url_x + j < viz_width:
            visualization[url_y][url_x + j] = char
        url_y += 2
      
      # Add QR code below URLs in compact mode
      if qr_lines and show_qr:
        # Center the QR code
        qr_start_x = max(0, (viz_width - qr_width) // 2)
        qr_start_y = url_y + 1
        
        for i, line in enumerate(qr_lines):
          if qr_start_y + i < term_height - 6:  # Leave room for GPU bar
            for j, char in enumerate(line):
              if 0 <= qr_start_x + j < viz_width:
                visualization[qr_start_y + i][qr_start_x + j] = char
        
        url_y = qr_start_y + len(qr_lines) + 1
    else:
      # Full mode: QR code on right, URLs on left
      qr_start_x = viz_width - qr_width - 5  # 5 chars margin from right
      qr_start_y = info_start_y
      
      if qr_lines:
        # Add QR label
        qr_label = "📱 Scan to join:"
        label_x = qr_start_x + (qr_width - len(qr_label)) // 2
        if label_x >= 0 and qr_start_y < term_height:
          for j, char in enumerate(qr_label):
            if label_x + j < viz_width:
              visualization[qr_start_y][label_x + j] = char
        
        # Add QR code
        for i, line in enumerate(qr_lines):
          if qr_start_y + i + 2 < term_height:  # +2 for label and spacing
            for j, char in enumerate(line):
              if qr_start_x + j < viz_width:
                visualization[qr_start_y + i + 2][qr_start_x + j] = char
      
      # Place URLs on the left side
      url_start_x = 5  # Left margin
      url_y = info_start_y
      max_url_x = qr_start_x - 2 if qr_lines else viz_width - 2
      
      if web_url:
        url_label = "Web Chat URL:"
        for j, char in enumerate(url_label):
          if url_start_x + j < max_url_x:
            visualization[url_y][url_start_x + j] = char
        url_y += 1
        
        for j, char in enumerate(web_url):
          if url_start_x + j < max_url_x:
            visualization[url_y][url_start_x + j] = char
        url_y += 2
      
      if api_url and not compact_mode:  # Skip API URL in compact mode
        api_label = "Chat API endpoint:"
        for j, char in enumerate(api_label):
          if url_start_x + j < max_url_x:
            visualization[url_y][url_start_x + j] = char
        url_y += 1
        
        for j, char in enumerate(api_url):
          if url_start_x + j < max_url_x:
            visualization[url_y][url_start_x + j] = char
        url_y += 2

    # Calculate where the GPU bar should go
    if mobile_mode or compact_mode:
      # Put GPU bar at bottom in mobile/compact mode
      bar_y = term_height - 4  # Closer to bottom for compact view
    else:
      # In full mode, account for QR code if present
      if qr_lines and 'qr_start_y' in locals():
        bar_y = max(url_y + 1, qr_start_y + len(qr_lines) + 4)
      else:
        bar_y = url_y + 1
    
    # Ensure bar_y is within bounds
    if bar_y >= term_height - 4:
      bar_y = term_height - 5  # Leave room for labels
    
    # Calculate total FLOPS and position on the bar
    total_flops = sum(self.topology.nodes.get(partition.node_id, UNKNOWN_DEVICE_CAPABILITIES).flops.fp16 for partition in self.partitions)
    bar_pos = (math.tanh(total_flops**(1/3)/2.5 - 2) + 1)

    # Add GPU poor/rich bar - adjust size for mobile
    if mobile_mode or compact_mode:
      # Compact bar for small screens
      bar_width = min(16, viz_width // 3)  # Smaller bar
    else:
      bar_width = min(30, viz_width // 3)
    
    bar_start_x = (viz_width - bar_width) // 2

    # Create a gradient bar using emojis
    gradient_bar = ""
    emojis = ["🟥", "🟧", "🟨", "🟩"]
    for i in range(bar_width):
      emoji_index = min(int(i/(bar_width/len(emojis))), len(emojis) - 1)
      gradient_bar += emojis[emoji_index]

    # Add the gradient bar to the visualization with brackets
    # Center the entire bar including brackets
    bar_full_width = len(gradient_bar) + 2  # +2 for brackets
    
    # For mobile/compact, use compact labels
    if mobile_mode or compact_mode:
      poor_label = "Poor"
      rich_label = "Rich"
    else:
      poor_label = "Poor"
      rich_label = "Rich"
    bar_start_x = (viz_width - bar_full_width) // 2
    
    # Add brackets
    if bar_start_x >= 0:
      visualization[bar_y][bar_start_x] = "["
    if bar_start_x + bar_full_width - 1 < viz_width:
      visualization[bar_y][bar_start_x + bar_full_width - 1] = "]"
    
    # Add the gradient bar (emojis)
    for i, char in enumerate(gradient_bar):
      if bar_start_x + 1 + i < viz_width:
        visualization[bar_y][bar_start_x + 1 + i] = char

    # Add labels (adjust for narrow terminals)
    if mobile_mode or compact_mode:
      # Compact layout: labels at edges of screen
      poor_label = "Poor"
      rich_label = "Rich"
      
      # Place labels at the very edges for compact view
      poor_start = 1  # Left edge
      rich_start = viz_width - len(rich_label) - 1  # Right edge
      
      # Add labels on same line as bar
      for i, char in enumerate(poor_label):
        if poor_start + i < bar_start_x - 1:  # Don't overlap with bar
          visualization[bar_y][poor_start + i] = char
      for i, char in enumerate(rich_label):
        if rich_start > bar_start_x + bar_full_width and rich_start + i < viz_width:
          visualization[bar_y][rich_start + i] = char
    else:
      # Full layout: labels adjacent to bar
      poor_label = "GPU poor"
      rich_label = "GPU rich"
      
      # Position labels next to the bar
      poor_start = max(0, bar_start_x - len(poor_label) - 2)
      rich_start = min(bar_start_x + bar_full_width + 2, viz_width - len(rich_label))
      
      for i, char in enumerate(poor_label):
        if poor_start + i < viz_width:
          visualization[bar_y][poor_start + i] = char
      for i, char in enumerate(rich_label):
        if rich_start + i < viz_width:
          visualization[bar_y][rich_start + i] = char

    # Add position indicator and FLOPS value
    # Position is relative to the actual emoji bar (inside brackets)
    if not mobile_mode:  # Skip arrow indicators on very small screens
      pos_x = bar_start_x + 1 + int(bar_pos * len(gradient_bar))
      flops_str = f"{total_flops:.2f} TFLOPS"
      if viz_width < 80:
        flops_str = f"{total_flops:.1f}TF"  # Shorter format for narrow terminals
      
      # Place indicator and text safely
      if 0 <= pos_x < viz_width:
        visualization[bar_y - 1][pos_x] = "▼"
        visualization[bar_y + 2][pos_x] = "▲"
      
      # Center FLOPS text
      flops_start = max(0, pos_x - len(flops_str) // 2)
      flops_end = min(viz_width, flops_start + len(flops_str))
      for i, char in enumerate(flops_str):
        if flops_start + i < flops_end:
          visualization[bar_y + 1][flops_start + i] = char
    else:
      # Mobile mode: just show FLOPS value below bar
      flops_str = f"{total_flops:.1f}TF"
      flops_start = (viz_width - len(flops_str)) // 2
      for i, char in enumerate(flops_str):
        if flops_start + i < viz_width and bar_y + 1 < term_height:
          visualization[bar_y + 1][flops_start + i] = char

    # Add an extra empty line for spacing
    bar_y += 4
    
    # Define the network visualization area
    # In mobile/compact mode, QR is below so use full width
    # In full mode, avoid QR on right if present
    if mobile_mode or compact_mode or not qr_lines:
      network_max_x = viz_width - 5
    else:
      # QR is on the right in full mode
      network_max_x = qr_start_x - 5 if 'qr_start_x' in locals() else viz_width - 5
    
    # Adjust center and radius for network visualization to fit in available space
    network_center_x = network_max_x // 2
    network_center_y = bar_y + 8  # Below the GPU bar
    network_radius_x = min(radius_x, (network_max_x - 10) // 2)
    network_radius_y = min(radius_y, 8)

    for i, partition in enumerate(self.partitions):
      device_capabilities = self.topology.nodes.get(partition.node_id, UNKNOWN_DEVICE_CAPABILITIES)

      angle = 2*math.pi*i/num_partitions
      x = int(network_center_x + network_radius_x*math.cos(angle))
      y = int(network_center_y + network_radius_y*math.sin(angle))

      # Place node with different color for active node and this node
      # Check bounds before placing node
      if 0 <= y < term_height and 0 <= x < viz_width:
        if partition.node_id == self.topology.active_node_id:
          visualization[y][x] = "🔴"
        elif partition.node_id == self.node_id:
          visualization[y][x] = "🟢"
        else:
          visualization[y][x] = "🔵"

      # Place node info (model, memory, TFLOPS, partition) on three lines
      # Adjust format based on terminal width
      if viz_width < 80:
        # Compact format for narrow terminals
        model_name = device_capabilities.model.split()[-1][:10]  # Shortened model name
        node_info = [
          f"{model_name} {device_capabilities.memory // 1024}GB",
          f"{device_capabilities.flops.fp16:.1f}TF",
          f"[{partition.start:.1f}-{partition.end:.1f}]",
        ]
      else:
        # Full format for wider terminals
        node_info = [
          f"{device_capabilities.model} {device_capabilities.memory // 1024}GB",
          f"{device_capabilities.flops.fp16}TFLOPS",
          f"[{partition.start:.2f}-{partition.end:.2f}]",
        ]

      # Calculate info position based on angle
      info_distance_x = network_radius_x + 6
      info_distance_y = network_radius_y + 3
      info_x = int(network_center_x + info_distance_x*math.cos(angle))
      info_y = int(network_center_y + info_distance_y*math.sin(angle))

      # Adjust text position to avoid overwriting the node icon and prevent cutoff
      max_info_len = len(max(node_info, key=len))
      # Ensure we don't overlap with QR code area
      max_allowed_x = network_max_x - max_info_len - 1
      if info_x < x:
        info_x = max(0, x - max_info_len - 1)
      elif info_x > x:
        info_x = min(max_allowed_x, info_x)

      # Adjust for top and bottom nodes
      if 5*math.pi/4 < angle < 7*math.pi/4:
        info_x = min(info_x + 4, max_allowed_x)
      elif math.pi/4 < angle < 3*math.pi/4:
        info_x = min(info_x + 3, max_allowed_x)
        info_y -= 2

      for j, line in enumerate(node_info):
        for k, char in enumerate(line):
          if 0 <= info_y + j < term_height and 0 <= info_x + k < viz_width:
            if info_y + j != y or info_x + k != x:
              visualization[info_y + j][info_x + k] = char

      # Draw line to next node and add connection description
      next_i = (i+1) % num_partitions
      next_angle = 2*math.pi*next_i/num_partitions
      next_x = int(network_center_x + network_radius_x*math.cos(next_angle))
      next_y = int(network_center_y + network_radius_y*math.sin(next_angle))

      # Get connection descriptions
      conn1 = self.topology.peer_graph.get(partition.node_id, set())
      conn2 = self.topology.peer_graph.get(self.partitions[next_i].node_id, set())
      description1 = next((c.description for c in conn1 if c.to_id == self.partitions[next_i].node_id), "")
      description2 = next((c.description for c in conn2 if c.to_id == partition.node_id), "")
      connection_description = f"{description1}/{description2}"

      # Simple line drawing
      steps = max(abs(next_x - x), abs(next_y - y))
      for step in range(1, steps):
        line_x = int(x + (next_x-x)*step/steps)
        line_y = int(y + (next_y-y)*step/steps)
        # Don't draw lines in the QR code area
        if 0 <= line_y < term_height and 0 <= line_x < network_max_x:
          visualization[line_y][line_x] = "-"

      # Add connection description near the midpoint of the line
      mid_x = (x + next_x) // 2
      mid_y = (y + next_y) // 2
      # Center the description text around the midpoint
      desc_start_x = mid_x - len(connection_description) // 2
      # Ensure description doesn't go into QR area
      desc_end_x = min(desc_start_x + len(connection_description), network_max_x)
      for j, char in enumerate(connection_description):
        if 0 <= mid_y < term_height and desc_start_x + j < desc_end_x:
          visualization[mid_y][desc_start_x + j] = char

    # Convert to string
    return "\n".join("".join(str(char) for char in row) for row in visualization)
  
  def _get_version_string(self) -> str:
    """Get the version string for display."""
    try:
      import importlib.metadata
      try:
        hanzo_net_version = importlib.metadata.version('hanzo-net')
      except:
        hanzo_net_version = "dev"
      return hanzo_net_version
    except:
      return "dev"

  def _generate_download_layout(self) -> Table:
    summary = Table(show_header=False, box=None, padding=(0, 1), expand=True)
    summary.add_column("Info", style="cyan", no_wrap=True, ratio=50)
    summary.add_column("Progress", style="cyan", no_wrap=True, ratio=40)
    summary.add_column("Percentage", style="cyan", no_wrap=True, ratio=10)

    # Current node download progress
    if self.node_id in self.node_download_progress:
      download_progress = self.node_download_progress[self.node_id]
      title = f"Downloading model {download_progress.repo_id}@{download_progress.repo_revision} ({download_progress.completed_files}/{download_progress.total_files}):"
      summary.add_row(Text(title, style="bold"))
      progress_info = f"{pretty_print_bytes(download_progress.downloaded_bytes)} / {pretty_print_bytes(download_progress.total_bytes)} ({pretty_print_bytes_per_second(download_progress.overall_speed)})"
      summary.add_row(progress_info)

      eta_info = f"{download_progress.overall_eta}"
      summary.add_row(eta_info)

      summary.add_row("")  # Empty row for spacing

      for file_path, file_progress in download_progress.file_progress.items():
        if file_progress.status != "complete":
          progress = int(file_progress.downloaded/file_progress.total*30)
          bar = f"[{'=' * progress}{' ' * (30 - progress)}]"
          percentage = f"{file_progress.downloaded / file_progress.total * 100:.0f}%"
          summary.add_row(Text(file_path[:30], style="cyan"), bar, percentage)

    summary.add_row("")  # Empty row for spacing

    # Other nodes download progress summary
    summary.add_row(Text("Other Nodes Download Progress:", style="bold"))
    for node_id, progress in self.node_download_progress.items():
      if node_id != self.node_id:
        device = self.topology.nodes.get(node_id)
        partition = next((p for p in self.partitions if p.node_id == node_id), None)
        partition_info = f"[{partition.start:.2f}-{partition.end:.2f}]" if partition else ""
        percentage = progress.downloaded_bytes/progress.total_bytes*100 if progress.total_bytes > 0 else 0
        speed = pretty_print_bytes_per_second(progress.overall_speed)
        device_info = f"{device.model if device else 'Unknown Device'} {device.memory // 1024 if device else '?'}GB {partition_info}"
        progress_info = f"{progress.repo_id}@{progress.repo_revision} ({speed})"
        progress_bar = f"[{'=' * int(percentage // 3.33)}{' ' * (30 - int(percentage // 3.33))}]"
        percentage_str = f"{percentage:.1f}%"
        eta_str = f"{progress.overall_eta}"
        summary.add_row(device_info, progress_info, percentage_str)
        summary.add_row("", progress_bar, eta_str)

    return summary
