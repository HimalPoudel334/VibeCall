
// document.getElementById('loginForm').addEventListener('submit', async function(e) {
//     e.preventDefault();
//
//     const username = document.getElementById('username').value.trim();
//     const password = document.getElementById('password').value;
//     const messageDiv = document.getElementById('message');
//
//     messageDiv.textContent = '';
//
//     try {
//         const response = await fetch('http://localhost:8085/auth/login', {
//             method: 'POST',
//             headers: {
//                 'Content-Type': 'application/json'
//             },
//             body: JSON.stringify({ username, password })
//         });
//
//         const data = await response.json();
//
//         if (response.ok) {
//             messageDiv.style.color = 'green';
//             messageDiv.textContent = 'Login successful!';
//         } else {
//             messageDiv.style.color = 'red';
//             messageDiv.textContent = data.message || 'Login failed!';
//         }
//     } catch (error) {
//         messageDiv.style.color = 'red';
//         messageDiv.textContent = 'An error occurred. Please try again.';
//         console.error('Login Error:', error);
//     }
// });
