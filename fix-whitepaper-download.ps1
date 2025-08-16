# Fix Whitepaper Download API
$ServerIP = "103.245.38.44"
$Username = "root"
$Password = "a6?#PMWdik52"

Write-Host "Fixing Whitepaper Download API..." -ForegroundColor Green

# Create a temporary file with the fixed route.ts content
$fixedRouteContent = @'
import { NextRequest, NextResponse } from "next/server";
import { readFile } from "fs/promises";
import { join } from "path";
import { existsSync } from "fs";

export async function GET(
  req: NextRequest,
  { params }: { params: { filename: string } }
) {
  try {
    const { filename } = params;
    
    // Security: Only allow specific whitepaper files
    const allowedFiles = [
      'WHITEPAPER_INDONESIA.md',
      'WHITEPAPER_ENGLISH.md',
      'README.md',
      'SECURITY.md'
    ];
    
    if (!allowedFiles.includes(filename)) {
      return NextResponse.json(
        { error: 'File not allowed' },
        { status: 403 }
      );
    }
    
    // Try multiple paths to find the file
    const possiblePaths = [
      join(process.cwd(), filename),                // Current working directory
      join('/opt/fvc-dashboard', filename),         // Root dashboard directory
      join('/var/www/dashboard', filename),         // Alternative dashboard location
      join('/opt/fvc-dashboard/public', filename),  // Public directory
      join(process.cwd(), '../', filename)          // One level up
    ];
    
    // Debug logging
    console.log('Current working directory:', process.cwd());
    console.log('Requested filename:', filename);
    console.log('Checking possible paths...');
    
    // Find the first path that exists
    let filePath = null;
    for (const path of possiblePaths) {
      console.log('Checking path:', path, existsSync(path));
      if (existsSync(path)) {
        filePath = path;
        break;
      }
    }
    
    // If no file found in any location
    if (!filePath) {
      return NextResponse.json(
        { error: `File not found: ${filename}. Checked paths: ${possiblePaths.join(', ')}` },
        { status: 404 }
      );
    }
    
    // Read file content
    const fileContent = await readFile(filePath, 'utf-8');
    
    // Determine content type based on file extension
    const contentType = filename.endsWith('.md') 
      ? 'text/markdown; charset=utf-8'
      : 'text/plain; charset=utf-8';
    
    // Return file content with appropriate headers
    return new NextResponse(fileContent, {
      status: 200,
      headers: {
        'Content-Type': contentType,
        'Content-Disposition': `inline; filename="${filename}"`,
        'Cache-Control': 'public, max-age=3600', // Cache for 1 hour
      },
    });
    
  } catch (error) {
    console.error('Error serving file:', error);
    return NextResponse.json(
      { error: 'Internal server error' },
      { status: 500 }
    );
  }
}

// Also support HEAD requests for checking file existence
export async function HEAD(
  req: NextRequest,
  { params }: { params: { filename: string } }
) {
  try {
    const { filename } = params;
    
    const allowedFiles = [
      'WHITEPAPER_INDONESIA.md',
      'WHITEPAPER_ENGLISH.md', 
      'README.md',
      'SECURITY.md'
    ];
    
    if (!allowedFiles.includes(filename)) {
      return new NextResponse(null, { status: 403 });
    }
    
    // Try multiple paths to find the file
    const possiblePaths = [
      join(process.cwd(), filename),                // Current working directory
      join('/opt/fvc-dashboard', filename),         // Root dashboard directory
      join('/var/www/dashboard', filename),         // Alternative dashboard location
      join('/opt/fvc-dashboard/public', filename),  // Public directory
      join(process.cwd(), '../', filename)          // One level up
    ];
    
    // Find the first path that exists
    let filePath = null;
    for (const path of possiblePaths) {
      if (existsSync(path)) {
        filePath = path;
        break;
      }
    }
    
    // If no file found in any location
    if (!filePath) {
      return new NextResponse(null, { status: 404 });
    }
    
    const contentType = filename.endsWith('.md')
      ? 'text/markdown; charset=utf-8'
      : 'text/plain; charset=utf-8';
    
    return new NextResponse(null, {
      status: 200,
      headers: {
        'Content-Type': contentType,
        'Content-Disposition': `inline; filename="${filename}"`,
        'Cache-Control': 'public, max-age=3600',
      },
    });
    
  } catch (error) {
    return new NextResponse(null, { status: 500 });
  }
}
'@

# Save the fixed route.ts to a temporary file
$tempFile = "route.ts.fixed"
Set-Content -Path $tempFile -Value $fixedRouteContent

# Upload the fixed route.ts file to the server
Write-Host "[INFO] Uploading fixed route.ts file..." -ForegroundColor Cyan
echo y | pscp -pw $Password $tempFile $Username@${ServerIP}:/opt/fvc-dashboard/src/app/api/download/[filename]/route.ts

# Remove the temporary file
Remove-Item -Path $tempFile

# Upload whitepaper files to multiple locations to ensure they're found
Write-Host "[INFO] Uploading whitepaper files to server..." -ForegroundColor Cyan

# List of whitepaper files to upload
$whitepaperFiles = @(
    "WHITEPAPER_INDONESIA.md",
    "WHITEPAPER_ENGLISH.md",
    "README.md",
    "SECURITY.md"
)

# Upload each whitepaper file to multiple locations
foreach ($file in $whitepaperFiles) {
    if (Test-Path $file) {
        Write-Host "[INFO] Uploading $file..." -ForegroundColor Cyan
        
        # Upload to root dashboard directory
        echo y | pscp -pw $Password $file $Username@${ServerIP}:/opt/fvc-dashboard/
        
        # Upload to public directory
        echo y | plink -ssh -pw $Password $Username@$ServerIP "mkdir -p /opt/fvc-dashboard/public"
        echo y | pscp -pw $Password $file $Username@${ServerIP}:/opt/fvc-dashboard/public/
    } else {
        Write-Host "[WARNING] File $file not found locally" -ForegroundColor Yellow
    }
}

# Rebuild the dashboard on server
Write-Host "[INFO] Rebuilding dashboard on server..." -ForegroundColor Cyan
echo y | plink -ssh -pw $Password $Username@$ServerIP "cd /opt/fvc-dashboard && npm run build"

# Restart the dashboard service
Write-Host "[INFO] Restarting dashboard service..." -ForegroundColor Cyan
echo y | plink -ssh -pw $Password $Username@$ServerIP "systemctl restart fvc-dashboard"

# Verify the fix
Write-Host "[INFO] Verifying the fix..." -ForegroundColor Cyan
echo y | plink -ssh -pw $Password $Username@$ServerIP "curl -s http://localhost:3001/api/download/WHITEPAPER_INDONESIA.md | head -n 5"

Write-Host "\nFix for whitepaper download completed!" -ForegroundColor Green
Write-Host "You can verify the changes at: https://fvchain.xyz/api/download/WHITEPAPER_INDONESIA.md" -ForegroundColor Yellow