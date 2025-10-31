#!/usr/bin/env python3
"""
Simple HTTP server to provide position data and AI decisions for dashboard.
Reads from SQLite database and serves JSON endpoints.
"""
import json
import sqlite3
from http.server import HTTPServer, SimpleHTTPRequestHandler
from pathlib import Path
import time
from datetime import datetime

DB_PATH = "../pump-sniper-bot/sniper_bot.db"

class MonitorHandler(SimpleHTTPRequestHandler):
    def do_GET(self):
        if self.path == '/api/positions':
            self.serve_positions()
        elif self.path == '/api/ai-stream':
            self.serve_ai_stream()
        elif self.path == '/api/stats':
            self.serve_stats()
        elif self.path == '/' or self.path == '/index.html':
            self.path = '/dashboard.html'
            return SimpleHTTPRequestHandler.do_GET(self)
        else:
            return SimpleHTTPRequestHandler.do_GET(self)

    def do_POST(self):
        if self.path == '/api/control/start':
            self.send_control_response("Bot started")
        elif self.path == '/api/control/pause':
            self.send_control_response("Bot paused")
        elif self.path == '/api/control/sell-all':
            self.send_control_response("Emergency sell initiated")
        else:
            self.send_error(404)

    def serve_positions(self):
        """Get active positions with current P&L"""
        try:
            conn = sqlite3.connect(DB_PATH)
            cursor = conn.cursor()

            cursor.execute("""
                SELECT
                    mint,
                    entry_sol_amount,
                    entry_time,
                    current_token_amount,
                    profit_loss_sol,
                    profit_loss_percent
                FROM positions
                WHERE status = 'active'
                ORDER BY entry_time DESC
                LIMIT 3
            """)

            positions = []
            for row in cursor.fetchall():
                mint, entry_sol, entry_time, tokens, pnl_sol, pnl_pct = row

                positions.append({
                    'mint': mint[:8] + '...' + mint[-8:],
                    'full_mint': mint,
                    'entry_sol': round(entry_sol, 3),
                    'entry_time': datetime.fromtimestamp(entry_time).strftime('%H:%M:%S'),
                    'tokens': int(tokens),
                    'pnl_sol': round(pnl_sol or 0, 3),
                    'pnl_percent': round(pnl_pct or 0, 1),
                    'current_value': round((entry_sol + (pnl_sol or 0)), 3)
                })

            conn.close()

            self.send_json_response(positions)
        except Exception as e:
            print(f"Error serving positions: {e}")
            self.send_json_response([])

    def serve_ai_stream(self):
        """Get recent AI decisions and reasoning"""
        try:
            conn = sqlite3.connect(DB_PATH)
            cursor = conn.cursor()

            cursor.execute("""
                SELECT
                    mint,
                    action,
                    confidence,
                    reasoning,
                    timestamp
                FROM ai_decisions
                ORDER BY timestamp DESC
                LIMIT 20
            """)

            decisions = []
            for row in cursor.fetchall():
                mint, action, confidence, reasoning, ts = row

                decisions.append({
                    'mint': mint[:8] + '...',
                    'action': action,
                    'confidence': round(confidence, 2),
                    'reasoning': reasoning,
                    'time': datetime.fromtimestamp(ts).strftime('%H:%M:%S')
                })

            conn.close()

            self.send_json_response(decisions)
        except Exception as e:
            print(f"Error serving AI stream: {e}")
            self.send_json_response([])

    def serve_stats(self):
        """Get overall statistics"""
        try:
            conn = sqlite3.connect(DB_PATH)
            cursor = conn.cursor()

            # Total P&L from active positions
            cursor.execute("""
                SELECT
                    SUM(profit_loss_sol) as total_pnl,
                    COUNT(*) as active_count
                FROM positions
                WHERE status = 'active'
            """)
            total_pnl, active_count = cursor.fetchone()

            # Unrealized P&L (same as total for active positions)
            unrealized_pnl = total_pnl or 0

            # Win rate from closed positions
            cursor.execute("""
                SELECT
                    COUNT(*) as total_closed,
                    SUM(CASE WHEN profit_loss_sol > 0 THEN 1 ELSE 0 END) as wins
                FROM positions
                WHERE status = 'closed'
            """)
            total_closed, wins = cursor.fetchone()

            win_rate = (wins / total_closed * 100) if total_closed else 0

            conn.close()

            stats = {
                'total_pnl': round(unrealized_pnl, 3),
                'unrealized_pnl': round(unrealized_pnl, 3),
                'active_positions': active_count or 0,
                'win_rate': round(win_rate, 1),
                'total_closed': total_closed or 0
            }

            self.send_json_response(stats)
        except Exception as e:
            print(f"Error serving stats: {e}")
            self.send_json_response({
                'total_pnl': 0,
                'unrealized_pnl': 0,
                'active_positions': 0,
                'win_rate': 0,
                'total_closed': 0
            })

    def send_json_response(self, data):
        """Send JSON response"""
        self.send_response(200)
        self.send_header('Content-Type', 'application/json')
        self.send_header('Access-Control-Allow-Origin', '*')
        self.end_headers()
        self.wfile.write(json.dumps(data).encode())

    def send_control_response(self, message):
        """Send control command response"""
        self.send_json_response({'status': 'ok', 'message': message})

def run_server(port=8080):
    """Run the monitoring server"""
    server_address = ('', port)
    httpd = HTTPServer(server_address, MonitorHandler)
    print(f"🚀 Monitor server running at http://localhost:{port}")
    print(f"📊 Dashboard: http://localhost:{port}/dashboard.html")
    print(f"🗄️  Database: {DB_PATH}")
    httpd.serve_forever()

if __name__ == '__main__':
    # Change to monitor directory to serve static files
    import os
    os.chdir(Path(__file__).parent)
    run_server(8080)
